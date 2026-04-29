#!/usr/bin/env bash
# Boot the main timeline server + all six tier-1 plugins in a sandboxed
# layout under tests/e2e/_run/. Synthesizes the on-disk fixtures the
# file-backed plugins (notification/usage/git/media_scan) need.
#
# Modes:
#   no args              foreground; tail logs and block. SIGINT shuts
#                        the whole stack down via the trap. Used for
#                        manual UI inspection.
#   --probe              boot, run a curl-based event-injection +
#                        round-trip verification, shut everything down,
#                        exit. Used by CI / smoke runs.
#
# Both modes assume `tests/e2e/build_tier1.sh` has already produced the
# six plugin server binaries and trunk dist outputs.

set -euo pipefail

cd "$(dirname "$0")/../.."
ROOT="$PWD"
SANDBOX="$ROOT/tests/e2e/_run"
LOG_DIR="$SANDBOX/logs"
DATA_DIR="$SANDBOX/data"
FIXTURES_DIR="$SANDBOX/fixtures"
RUN_DIR="$SANDBOX/runtime"

MAIN_PORT=18002
PASSWORD="smoke-test-pwd"

# Tier-1 plugin map: name -> port
declare -A PORTS=(
  [timeline_plugin_text]=19006
  [timeline_plugin_web]=19004
  [timeline_plugin_notification]=19007
  [timeline_plugin_usage]=19008
  [timeline_plugin_git]=19010
  [timeline_plugin_media_scan]=19005
)

# Per-plugin shared bearer token (regenerated each run for hygiene).
declare -A TOKENS

mode=foreground
for arg in "$@"; do
  case "$arg" in
    --probe) mode=probe ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

# ---------- cleanup ----------

declare -A PIDS=()
cleanup() {
  echo
  for name in "${!PIDS[@]}"; do
    pid="${PIDS[$name]}"
    if kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
    fi
  done
  for name in "${!PIDS[@]}"; do
    pid="${PIDS[$name]}"
    wait "$pid" 2>/dev/null || true
  done
}
trap cleanup EXIT INT TERM

# ---------- sandbox ----------

rm -rf "$SANDBOX/runtime" "$SANDBOX/logs" "$SANDBOX/data" "$SANDBOX/fixtures"
mkdir -p "$LOG_DIR" "$DATA_DIR/plugin_web" "$RUN_DIR" "$FIXTURES_DIR"

# Symlink frontend dist where the main server expects (../frontend/dist
# relative to its CWD).
mkdir -p "$RUN_DIR/main/frontend"
ln -sfn "$ROOT/frontend/dist" "$RUN_DIR/main/frontend/dist"
mkdir -p "$RUN_DIR/main/server"

# ---------- fixtures ----------

# notification + usage share apps_file + app_icons.
APPS_FILE="$FIXTURES_DIR/apps"
APP_ICONS_DIR="$FIXTURES_DIR/app_icons"
mkdir -p "$APP_ICONS_DIR"
cat > "$APPS_FILE" <<EOF
com.test.alpha:Test Alpha
com.test.beta:Test Beta
EOF
# Tiny 1x1 transparent PNG as a fake app icon.
echo "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==" \
  | base64 -d > "$APP_ICONS_DIR/com.test.alpha"
cp "$APP_ICONS_DIR/com.test.alpha" "$APP_ICONS_DIR/com.test.beta"

# usage: a single per-day file with hourly open/lock pairs.
USAGE_DIR="$FIXTURES_DIR/usage_files"
mkdir -p "$USAGE_DIR"
NOW_TS=$(date -u +%s)
DAY_START_TS=$(( (NOW_TS / 86400) * 86400 ))
USAGE_FILE="$USAGE_DIR/$DAY_START_TS"
cat > "$USAGE_FILE" <<EOF
$(( DAY_START_TS + 3600 )):open:com.test.alpha
$(( DAY_START_TS + 4200 )):lock:
$(( DAY_START_TS + 7200 )):open:com.test.beta
$(( DAY_START_TS + 8400 )):lock:
EOF

# git: a tmp repo with one commit dated today.
GIT_ROOT="$FIXTURES_DIR/git_root"
mkdir -p "$GIT_ROOT/sample_repo"
( cd "$GIT_ROOT/sample_repo"
  git init -q
  git config user.email "test@example.com"
  git config user.name "Test User"
  echo hello > README.md
  git add README.md
  git commit -q -m "smoke test commit"
)

# media_scan: a tmp dir with a tiny jpeg.
MEDIA_DIR="$FIXTURES_DIR/media"
mkdir -p "$MEDIA_DIR"
# Smallest valid 1x1 JPEG, base64-decoded for safety.
echo "/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAAgGBgcGBQgHBwcJCQgKDBQNDAsLDBkSEw8UHRofHh0aHBwgJC4nICIsIxwcKDcpLDAxNDQ0Hyc5PTgyPC4zNDL/wAALCAABAAEBAREA/9sAQwEJCQkMCwwYDQ0YMiEcITIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIy/8AAEQgAAQABAwEiAAIRAQMRAf/EAB8AAAEFAQEBAQEBAAAAAAAAAAABAgMEBQYHCAkKC//EALUQAAIBAwMCBAMFBQQEAAABfQECAwAEEQUSITFBBhNRYQciMnGBFEKRobHBCSMzUvAVYnLRChYkNOEl8RcYGRolJicoKSo1Njc4OTpDREVGR0hJSlNUVVZXWFlaY2RlZmdoaWpzdHV2d3h5eoOEhYaHiImKkpOUlZaXmJmaoqOkpaanqKmqsrO0tba3uLm6wsPExcbHyMnK0tPU1dbX2Nna4eLj5OXm5+jp6vHy8/T19vf4+fr/2gAIAQEAAD8A+8KKKKKK/9k=" \
  | base64 -d > "$MEDIA_DIR/sample.jpg"

# ---------- per-plugin configs ----------

declare -A NOTIFY_PWD=()
declare -A WEB_TOKEN=()

for plugin in "${!PORTS[@]}"; do
  port=${PORTS[$plugin]}
  token=$(head -c 32 /dev/urandom | base64 | tr -d '/+=' | head -c 24)
  TOKENS[$plugin]=$token

  cwd="$RUN_DIR/$plugin"
  mkdir -p "$cwd"

  # Per-plugin [config] block content.
  case "$plugin" in
    timeline_plugin_text)
      extra='[config]'
      ;;
    timeline_plugin_web)
      extra='[config]'
      ;;
    timeline_plugin_notification)
      pwd_n=$(head -c 16 /dev/urandom | base64 | tr -d '/+=' | head -c 16)
      NOTIFY_PWD[$plugin]=$pwd_n
      extra="[config]
apps_file = \"$APPS_FILE\"
app_icon_files = \"$APP_ICONS_DIR\"
notification_password = \"$pwd_n\""
      ;;
    timeline_plugin_usage)
      extra="[config]
usage_files = \"$USAGE_DIR\"
apps_file = \"$APPS_FILE\"
app_icon_files = \"$APP_ICONS_DIR\""
      ;;
    timeline_plugin_git)
      extra="[config]
repo_folder = \"$GIT_ROOT\""
      ;;
    timeline_plugin_media_scan)
      extra="[config]
interval = 1
[config.locations.SmokeTest]
location = \"$MEDIA_DIR\""
      ;;
  esac

  cat > "$cwd/config.toml" <<EOF
[plugin]
name = "$plugin"
port = $port
token = "$token"
data_dir = "$DATA_DIR"

$extra
EOF
done

# ---------- main server config ----------

{
  echo "port = $MAIN_PORT"
  echo "password = \"$PASSWORD\""
  echo "data_dir = \"$DATA_DIR\""
  echo
  for plugin in "${!PORTS[@]}"; do
    echo "[[plugin]]"
    echo "name = \"$plugin\""
    echo "url = \"http://127.0.0.1:${PORTS[$plugin]}\""
    echo "token = \"${TOKENS[$plugin]}\""
    echo
  done
} > "$RUN_DIR/main/server/config.toml"

# ---------- copy plugin web bundles into the data_dir ----------

for plugin in "${!PORTS[@]}"; do
  src="$ROOT/plugins/$plugin/client/dist"
  dst="$DATA_DIR/plugin_web/$plugin"
  rm -rf "$dst"
  cp -r "$src" "$dst"
done

# ---------- launch ----------

wait_for_ready() {
  local desc=$1 url=$2
  for _ in $(seq 1 60); do
    if curl -sf -o /dev/null -m 1 "$url"; then
      echo "  ready: $desc"
      return 0
    fi
    sleep 0.5
  done
  echo "FAIL: $desc never came up at $url" >&2
  return 1
}

echo "==> launching tier-1 plugins"
for plugin in "${!PORTS[@]}"; do
  port=${PORTS[$plugin]}
  bin="$ROOT/plugins/$plugin/server/target/release/${plugin}_server"
  cwd="$RUN_DIR/$plugin"
  log="$LOG_DIR/$plugin.log"
  ( cd "$cwd" && RUST_LOG=info exec "$bin" > "$log" 2>&1 ) &
  PIDS[$plugin]=$!
done

# Each plugin's /health is bearer-protected; check /metrics-style by
# hitting a TCP probe instead.
for plugin in "${!PORTS[@]}"; do
  port=${PORTS[$plugin]}
  for _ in $(seq 1 60); do
    if (echo > /dev/tcp/127.0.0.1/$port) 2>/dev/null; then
      echo "  ready: $plugin :$port"
      break
    fi
    sleep 0.5
  done
done

echo "==> launching main server :$MAIN_PORT"
MAIN_BIN="$ROOT/server/target/release/server"
test -x "$MAIN_BIN" || ( cd "$ROOT/server" && cargo +stable build --release --quiet )
( cd "$RUN_DIR/main/server" && RUST_LOG=info exec "$MAIN_BIN" > "$LOG_DIR/main.log" 2>&1 ) &
PIDS[main]=$!
wait_for_ready "main /api/auth" "http://127.0.0.1:$MAIN_PORT/index.html"

# Persist runtime metadata so external scripts (the playwright spec)
# can pick up port + password + per-plugin notification password.
{
  echo "MAIN_PORT=$MAIN_PORT"
  echo "PASSWORD=$PASSWORD"
  for p in "${!PORTS[@]}"; do
    echo "PORT_$p=${PORTS[$p]}"
  done
  for p in "${!NOTIFY_PWD[@]}"; do
    echo "NOTIFY_PWD_$p=${NOTIFY_PWD[$p]}"
  done
} > "$RUN_DIR/env"

echo "==> stack up. main http://127.0.0.1:$MAIN_PORT  cookie pwd=$PASSWORD"
echo "    runtime env: $RUN_DIR/env"

if [[ $mode == probe ]]; then
  echo "==> running event-injection probes"
  # Run as a subprocess so the cleanup trap still fires when we exit.
  "$ROOT/tests/e2e/probe_tier1.sh"
  exit $?
fi

echo "==> Ctrl-C to stop."
# Block until any PID dies (or we're killed).
wait -n
