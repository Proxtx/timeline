#!/usr/bin/env bash
# Boot the main timeline server + steam + spotify plugins for SUPERVISED
# end-to-end testing. These plugins poll external APIs (steam Web API,
# spotify Web API) — they only emit events while the user is actively
# playing a game / listening to music. The script reads credentials out
# of repo-root/old_config.toml.
#
# Usage:
#   ./tests/e2e/serve_tier3.sh           # foreground; tail logs, Ctrl-C to stop
#   ./tests/e2e/serve_tier3.sh --probe   # poll /api/events for up to 5 min,
#                                          succeed when a steam *or* spotify
#                                          event lands.

set -euo pipefail

cd "$(dirname "$0")/../.."
ROOT="$PWD"
REPO_ROOT="$(cd "$ROOT/.." && pwd)"
OLD_CFG="$REPO_ROOT/old_config.toml"

SANDBOX="$ROOT/tests/e2e/_run3"
LOG_DIR="$SANDBOX/logs"
DATA_DIR="$SANDBOX/data"
RUN_DIR="$SANDBOX/runtime"

MAIN_PORT=18003
PASSWORD="tier3-pwd"

declare -A PORTS=(
  [timeline_plugin_steam]=19011
  [timeline_plugin_spotify]=19012
)

declare -A TOKENS

mode=foreground
for arg in "$@"; do
  case "$arg" in
    --probe) mode=probe ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

# ---------- credentials ----------

test -f "$OLD_CFG" || { echo "missing $OLD_CFG" >&2; exit 1; }

extract_field() {
  # extract_field <section> <key>: returns the toml value (string, no quotes)
  awk -v s="[plugin_config.$1]" -v k="\"$2\"" '
    $0 == s {in_s=1; next}
    /^\[/ {in_s=0}
    in_s && $1 == k {
      sub(/^[^=]*=[ ]*/, "")
      gsub(/^"/, ""); gsub(/"$/, "")
      print
      exit
    }
  ' "$OLD_CFG"
}

STEAM_API_KEY=$(extract_field timeline_plugin_steam api_key)
STEAM_USER_ID=$(extract_field timeline_plugin_steam user_steam_id)
SPOTIFY_CLIENT_ID=$(extract_field timeline_plugin_spotify client_id)
SPOTIFY_CLIENT_SECRET=$(extract_field timeline_plugin_spotify client_secret)
SPOTIFY_REFRESH_TOKEN=$(extract_field timeline_plugin_spotify refresh_token)

for v in STEAM_API_KEY STEAM_USER_ID SPOTIFY_CLIENT_ID SPOTIFY_CLIENT_SECRET SPOTIFY_REFRESH_TOKEN; do
  if [[ -z "${!v:-}" ]]; then
    echo "FATAL: $v missing from $OLD_CFG" >&2
    exit 1
  fi
done
echo "==> credentials loaded"

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

rm -rf "$SANDBOX/runtime" "$SANDBOX/logs" "$SANDBOX/data"
mkdir -p "$LOG_DIR" "$DATA_DIR/plugin_web" "$RUN_DIR"
mkdir -p "$RUN_DIR/main/frontend"
ln -sfn "$ROOT/frontend/dist" "$RUN_DIR/main/frontend/dist"
mkdir -p "$RUN_DIR/main/server"

# ---------- per-plugin configs ----------

for plugin in "${!PORTS[@]}"; do
  port=${PORTS[$plugin]}
  token=$(head -c 32 /dev/urandom | base64 | tr -d '/+=' | head -c 24)
  TOKENS[$plugin]=$token

  cwd="$RUN_DIR/$plugin"
  mkdir -p "$cwd"

  case "$plugin" in
    timeline_plugin_steam)
      extra="[config]
api_key = \"$STEAM_API_KEY\"
user_steam_id = \"$STEAM_USER_ID\""
      ;;
    timeline_plugin_spotify)
      extra="[config]
client_id = \"$SPOTIFY_CLIENT_ID\"
client_secret = \"$SPOTIFY_CLIENT_SECRET\"
refresh_token = \"$SPOTIFY_REFRESH_TOKEN\""
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

# ---------- main config ----------

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

# ---------- copy plugin web bundles ----------

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

echo "==> launching tier-3 plugins"
for plugin in "${!PORTS[@]}"; do
  port=${PORTS[$plugin]}
  bin="$ROOT/plugins/$plugin/server/target/release/${plugin}_server"
  test -x "$bin" || { echo "missing $bin (build first)" >&2; exit 1; }
  cwd="$RUN_DIR/$plugin"
  log="$LOG_DIR/$plugin.log"
  ( cd "$cwd" && RUST_LOG=info exec "$bin" > "$log" 2>&1 ) &
  PIDS[$plugin]=$!
done

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
wait_for_ready "main" "http://127.0.0.1:$MAIN_PORT/index.html"

{
  echo "MAIN_PORT=$MAIN_PORT"
  echo "PASSWORD=$PASSWORD"
} > "$RUN_DIR/env"

echo
echo "==> tier-3 stack up"
echo "    main:    http://127.0.0.1:$MAIN_PORT  cookie pwd=$PASSWORD"
echo "    logs:    $LOG_DIR/{main,timeline_plugin_steam,timeline_plugin_spotify}.log"
echo "    runtime: $RUN_DIR/env"
echo

if [[ $mode == probe ]]; then
  exec "$ROOT/tests/e2e/probe_tier3.sh"
fi

echo "==> Ctrl-C to stop."
wait -n
