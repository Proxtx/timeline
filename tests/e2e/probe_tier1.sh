#!/usr/bin/env bash
# Drive each tier-1 plugin end-to-end through the main server's proxy:
# inject events where appropriate, then verify they round-trip via
# /api/events. Run by serve_tier1.sh --probe.
set -euo pipefail

cd "$(dirname "$0")/../.."
ROOT="$PWD"
RUN_DIR="$ROOT/tests/e2e/_run/runtime"
test -f "$RUN_DIR/env" || { echo "no runtime env file at $RUN_DIR/env" >&2; exit 2; }
# shellcheck disable=SC1091
source "$RUN_DIR/env"

BASE="http://127.0.0.1:$MAIN_PORT"
COOKIE="pwd=$PASSWORD"

ok() { echo "ok [$1]"; }
fail() { echo "FAIL [$1]: $2" >&2; exit 1; }

# ---------- inject events ----------

# text plugin: POST /create with timing covering "now". The playwright
# spec drives /event/latest which fetches events in the last hour (no
# timezone-dependent day filtering), so a "now"-anchored event is the
# most reliable signal across timezones.
NOW_NS=$(( $(date -u +%s) * 1000000000 ))
SLOT_START_NS=$(( NOW_NS - 30 * 1000000000 ))   # -30s
SLOT_END_NS=$((   NOW_NS + 30 * 1000000000 ))   # +30s

text_payload=$(cat <<EOF
{
  "text": "smoke test $(date -u +%FT%TZ)",
  "timing": [$SLOT_START_NS, $SLOT_END_NS]
}
EOF
)

curl -sf -X POST "$BASE/api/plugin/timeline_plugin_text/create" \
  -H "Content-Type: application/json" \
  -b "$COOKIE" \
  -d "$text_payload" >/dev/null \
  || fail "text create" "POST /create failed"
ok "text: /create"

# web plugin: POST /register_visit. The plugin then fetches OG metadata
# from the upstream URL (we use example.com). Tolerate a slow first
# fetch by giving it a few seconds.
visit_payload='{"client":"smoke","website":"https://example.com/"}'
if curl -sf -X POST "$BASE/api/plugin/timeline_plugin_web/register_visit" \
    -H "Content-Type: application/json" \
    -b "$COOKIE" \
    --max-time 20 \
    -d "$visit_payload" >/dev/null; then
  ok "web: /register_visit"
else
  echo "  warn: web /register_visit failed (no internet?). continuing."
fi

# notification plugin: GET /notification/<password>/<app>/<title>/<content>
notify_pwd_var="NOTIFY_PWD_timeline_plugin_notification"
notify_pwd="${!notify_pwd_var}"
notify_url="$BASE/api/plugin/timeline_plugin_notification/notification/$notify_pwd/com.test.alpha/SmokeTitle/SmokeContent"
curl -sf -b "$COOKIE" "$notify_url" >/dev/null \
  || fail "notification" "GET notification failed"
ok "notification: /notification/<pwd>/.../"

# usage / git / media_scan: events synthesized from the on-disk fixtures
# the orchestrator already laid down. No injection needed; just give the
# plugins' request_loops a moment to scan.
echo "  waiting 3s for usage/git/media_scan to settle..."
sleep 3

# ---------- verify ----------

day_start=$(date -u +%Y-%m-%dT00:00:00Z)
day_end=$(date -u -d "tomorrow" +%Y-%m-%dT00:00:00Z 2>/dev/null \
  || date -u -v+1d +%Y-%m-%dT00:00:00Z)

events=$(curl -sf -X POST "$BASE/api/events" \
  -H "Content-Type: application/json" \
  -b "$COOKIE" \
  -d "{\"start\":\"$day_start\",\"end\":\"$day_end\"}")

if [[ -z "$events" ]]; then
  fail "events" "empty response from /api/events"
fi

assert_present() {
  local plugin=$1
  if ! grep -q "\"$plugin\"" <<<"$events"; then
    fail "events" "plugin $plugin missing from /api/events response"
  fi
  ok "events: $plugin present in fan-out"
}

assert_present timeline_plugin_text
# web is best-effort given the OG fetch can fail offline
if grep -q '"timeline_plugin_web"' <<<"$events"; then
  ok "events: timeline_plugin_web present in fan-out"
else
  echo "  warn: web missing from fan-out (probably offline)"
fi
assert_present timeline_plugin_notification
assert_present timeline_plugin_usage
assert_present timeline_plugin_git
assert_present timeline_plugin_media_scan

# ---------- per-plugin sanity ----------

# Verify each plugin's manifest aggregates correctly.
manifests=$(curl -sf -X POST "$BASE/api/plugins" -b "$COOKIE")
for plugin in timeline_plugin_text timeline_plugin_web \
              timeline_plugin_notification timeline_plugin_usage \
              timeline_plugin_git timeline_plugin_media_scan; do
  if ! grep -q "\"$plugin\"" <<<"$manifests"; then
    fail "manifests" "$plugin missing from /api/plugins"
  fi
done
ok "manifests: all six plugins aggregated"

# Verify static plugin web bundles served.
for plugin in timeline_plugin_text timeline_plugin_web \
              timeline_plugin_notification timeline_plugin_usage \
              timeline_plugin_git timeline_plugin_media_scan; do
  url="$BASE/plugin_web/$plugin/${plugin}_client.js"
  status=$(curl -s -o /dev/null -w "%{http_code}" "$url")
  if [[ "$status" != "200" ]]; then
    fail "plugin_web" "$plugin js bundle returned $status (expected 200)"
  fi
done
ok "plugin_web: all six wasm bundles served"

# Cover image asset for media_scan: the file's modification time is
# recent so the scanner has indexed it. The asset is served via the
# proxy.
# (We just assert the events list contains a media_scan entry above;
#  serving the file requires a signature embedded in the event payload.)

echo
echo "==> all tier-1 probes passed"
