#!/usr/bin/env bash
# Build the six tier-1 plugins (server binaries + trunk client bundles).
# Idempotent — skips already-built artifacts unless --clean is passed.
set -euo pipefail

cd "$(dirname "$0")/../.."
ROOT="$PWD"

clean=0
for arg in "$@"; do
  case "$arg" in
    --clean) clean=1 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

PLUGINS=(
  timeline_plugin_text
  timeline_plugin_web
  timeline_plugin_notification
  timeline_plugin_usage
  timeline_plugin_git
  timeline_plugin_media_scan
)

for plugin in "${PLUGINS[@]}"; do
  dir="$ROOT/plugins/$plugin"
  if [[ ! -d "$dir" ]]; then
    echo "missing plugin dir: $dir" >&2
    exit 3
  fi

  echo "==> $plugin: server"
  ( cd "$dir/server" && [[ $clean -eq 1 ]] && cargo clean || true
    cd "$dir/server" && cargo +stable build --release --quiet
  )

  echo "==> $plugin: client (trunk)"
  ( cd "$dir/client" && [[ $clean -eq 1 ]] && rm -rf dist target/wasm32-unknown-unknown/release || true
    cd "$dir/client" && trunk build --release >/dev/null
  )

  # Confirm both expected artifacts exist.
  bin="$dir/server/target/release/${plugin}_server"
  test -x "$bin" || { echo "server binary missing: $bin" >&2; exit 4; }
  test -d "$dir/client/dist" || { echo "client dist missing for $plugin" >&2; exit 5; }
done

echo "==> all six tier-1 plugins built"
