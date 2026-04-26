#!/usr/bin/env bash
# Smoke test the rebuilt timeline stack end-to-end on the host.
# Builds frontend + server, boots the server with a zero-plugin config,
# and probes the core API and SPA fallback. Exits non-zero on any
# failure. Designed to be safe to run repeatedly: it uses a sandbox
# data dir under tests/e2e/_run and shuts the server down on exit.
#
# Usage: ./smoke.sh [--skip-build]

set -euo pipefail

cd "$(dirname "$0")/../.."
ROOT="$PWD"
SANDBOX="$ROOT/tests/e2e/_run"
LOG="$SANDBOX/server.log"
PORT=18002
PASSWORD="smoke-test-pwd"

skip_build=0
for arg in "$@"; do
  case "$arg" in
    --skip-build) skip_build=1 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

cleanup() {
  if [[ -n "${SERVER_PID:-}" ]] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

mkdir -p "$SANDBOX"

# --- build ---

if [[ $skip_build -eq 0 ]]; then
  echo "==> trunk build (frontend, release)"
  ( cd frontend && trunk build --release >/dev/null )
  echo "==> cargo build (server, release)"
  ( cd server && cargo +stable build --release --quiet )
fi

SERVER_BIN="$ROOT/server/target/release/server"
test -x "$SERVER_BIN" || { echo "server binary not found at $SERVER_BIN" >&2; exit 3; }

# --- sandbox layout ---
# The server reads config.toml from CWD and serves frontend/dist via the
# relative path ../frontend/dist/. We mirror that layout under the sandbox.

mkdir -p "$SANDBOX/server"
cat > "$SANDBOX/server/config.toml" <<EOF
port = $PORT
password = "$PASSWORD"
data_dir = "./data"
EOF
mkdir -p "$SANDBOX/frontend"
ln -sfn "$ROOT/frontend/dist" "$SANDBOX/frontend/dist"

# --- run ---

echo "==> starting server on :$PORT (logs: $LOG)"
( cd "$SANDBOX/server" && ROCKET_PROFILE=release RUST_LOG=info \
  "$SERVER_BIN" > "$LOG" 2>&1 ) &
SERVER_PID=$!

# wait for boot, with a hard timeout
for _ in $(seq 1 60); do
  if grep -q "Rocket has launched" "$LOG" 2>/dev/null; then break; fi
  sleep 0.5
done
grep -q "Rocket has launched" "$LOG" || {
  echo "server failed to boot within 30s" >&2
  tail -50 "$LOG" >&2
  exit 4
}

# --- probes ---

base="http://127.0.0.1:$PORT"

assert_status() {
  local desc=$1 expected=$2 cmd=$3
  local got
  got=$(eval "$cmd")
  if [[ "$got" != "$expected" ]]; then
    echo "FAIL [$desc]: expected status=$expected, got=$got" >&2
    return 1
  fi
  echo "ok [$desc]"
}

assert_body() {
  local desc=$1 expected=$2 cmd=$3
  local got
  got=$(eval "$cmd")
  if [[ "$got" != "$expected" ]]; then
    echo "FAIL [$desc]: expected body=$expected, got=$got" >&2
    return 1
  fi
  echo "ok [$desc]"
}

# auth without cookie → AuthenticationError
assert_body "auth: no cookie" \
  '{"Err":"AuthenticationError"}' \
  "curl -s -X POST $base/api/auth"

# auth with valid cookie → Ok
assert_body "auth: valid cookie" \
  '{"Ok":null}' \
  "curl -s -X POST $base/api/auth -b 'pwd=$PASSWORD'"

# events fan-out, zero plugins → empty map
assert_body "events: no plugins" \
  '{"Ok":{}}' \
  "curl -s -X POST $base/api/events -H 'Content-Type: application/json' \
     -d '{\"start\":\"2026-04-25T00:00:00Z\",\"end\":\"2026-04-25T23:59:59Z\"}' \
     -b 'pwd=$PASSWORD'"

# plugins manifest, zero plugins → []
assert_body "plugins: no plugins" \
  '{"Ok":[]}' \
  "curl -s -X POST $base/api/plugins -b 'pwd=$PASSWORD'"

# SPA fallback for unknown path → 202 with index.html
assert_status "spa fallback: /timeline/" "202" \
  "curl -s -o /dev/null -w '%{http_code}' $base/timeline/"

assert_status "spa fallback: deep link" "202" \
  "curl -s -o /dev/null -w '%{http_code}' $base/timeline/2026-04-25T00:00:00Z"

# Static asset: index.html for the frontend
assert_status "static: /index.html" "200" \
  "curl -s -o /dev/null -w '%{http_code}' $base/index.html"

echo "==> all smoke probes passed"
