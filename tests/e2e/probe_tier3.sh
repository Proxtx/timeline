#!/usr/bin/env bash
# Poll /api/events every 15s for up to 5 minutes. Succeed as soon as a
# steam OR spotify event lands in the fan-out. Run as
# `./tests/e2e/serve_tier3.sh --probe` after starting a game / playing
# a song on Spotify.
set -euo pipefail

cd "$(dirname "$0")/../.."
ROOT="$PWD"
RUN_DIR="$ROOT/tests/e2e/_run3/runtime"
LOG_DIR="$ROOT/tests/e2e/_run3/logs"
test -f "$RUN_DIR/env" || { echo "no runtime env at $RUN_DIR/env" >&2; exit 2; }
# shellcheck disable=SC1091
source "$RUN_DIR/env"

BASE="http://127.0.0.1:$MAIN_PORT"
COOKIE="pwd=$PASSWORD"

day_start=$(date -u +%Y-%m-%dT00:00:00Z)
day_end=$(date -u -d "tomorrow" +%Y-%m-%dT00:00:00Z 2>/dev/null \
  || date -u -v+1d +%Y-%m-%dT00:00:00Z)

deadline=$(( $(date +%s) + 300 ))
attempt=0
steam_seen=0
spotify_seen=0

echo "==> waiting for steam or spotify events (up to 5 min)"
echo "    play a game or hit play on Spotify now"

while [[ $(date +%s) -lt $deadline ]]; do
  attempt=$((attempt + 1))
  body=$(curl -sf -X POST "$BASE/api/events" \
    -H "Content-Type: application/json" \
    -b "$COOKIE" \
    -d "{\"start\":\"$day_start\",\"end\":\"$day_end\"}" \
    || echo "")

  steam_count=$(jq -r '.Ok.timeline_plugin_steam | length // 0' <<<"$body" 2>/dev/null || echo 0)
  spotify_count=$(jq -r '.Ok.timeline_plugin_spotify | length // 0' <<<"$body" 2>/dev/null || echo 0)

  if [[ "$steam_count" -gt 0 && "$steam_seen" -eq 0 ]]; then
    echo "ok [steam]  $steam_count event(s) at attempt $attempt"
    jq -r '.Ok.timeline_plugin_steam[].title' <<<"$body" | sed 's/^/      - /'
    steam_seen=1
  fi
  if [[ "$spotify_count" -gt 0 && "$spotify_seen" -eq 0 ]]; then
    echo "ok [spotify] $spotify_count event(s) at attempt $attempt"
    jq -r '.Ok.timeline_plugin_spotify[].title' <<<"$body" | sed 's/^/      - /'
    spotify_seen=1
  fi

  if [[ "$steam_seen" -eq 1 || "$spotify_seen" -eq 1 ]]; then
    echo "==> success — at least one supervised plugin emitted events"
    if [[ "$steam_seen" -eq 1 && "$spotify_seen" -eq 1 ]]; then
      exit 0
    fi
    # Keep polling another 60s to give the second plugin a chance.
    second_deadline=$(( $(date +%s) + 60 ))
    if [[ $second_deadline -lt $deadline ]]; then
      deadline=$second_deadline
    fi
  fi

  printf "    attempt %2d steam=%s spotify=%s\n" "$attempt" "$steam_count" "$spotify_count"
  sleep 15
done

if [[ "$steam_seen" -eq 1 || "$spotify_seen" -eq 1 ]]; then
  exit 0
fi

echo "FAIL: no events from steam or spotify within 5 min" >&2
echo "  --- steam log tail ---"
tail -20 "$LOG_DIR/timeline_plugin_steam.log" >&2 || true
echo "  --- spotify log tail ---"
tail -20 "$LOG_DIR/timeline_plugin_spotify.log" >&2 || true
exit 1
