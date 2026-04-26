#!/usr/bin/env bash
# Boot the timeline server in the same sandbox `smoke.sh` uses, but
# block instead of running probes. Useful for driving the Playwright
# spec or for manual browser testing.
set -euo pipefail

cd "$(dirname "$0")/../.."
ROOT="$PWD"
SANDBOX="$ROOT/tests/e2e/_run"
LOG="$SANDBOX/server.log"
PORT=18002
PASSWORD="smoke-test-pwd"

SERVER_BIN="$ROOT/server/target/release/server"
test -x "$SERVER_BIN" || {
  echo "server binary not found at $SERVER_BIN" >&2
  echo "run: cd server && cargo +stable build --release" >&2
  exit 3
}

# Build frontend dist if missing (release mode).
test -d "$ROOT/frontend/dist" || (cd "$ROOT/frontend" && trunk build --release)

mkdir -p "$SANDBOX/server" "$SANDBOX/frontend"
cat > "$SANDBOX/server/config.toml" <<EOF
port = $PORT
password = "$PASSWORD"
data_dir = "./data"
EOF
ln -sfn "$ROOT/frontend/dist" "$SANDBOX/frontend/dist"

echo "==> serving timeline at http://127.0.0.1:$PORT"
echo "    cookie:   pwd=$PASSWORD"
echo "    log file: $LOG"
exec env -C "$SANDBOX/server" RUST_LOG=info "$SERVER_BIN"
