# Migrating from the pre-rework architecture

This document walks through moving an existing timeline deployment from
the old MongoDB / linker / nightly-rust setup onto the new
SQLite-per-plugin architecture. Each plugin has its own
`MIGRATION.md` with the per-row conversion spec; this is the
top-level orchestration plus the cross-cutting changes.

## Overview of the change

| Concern | Before | After |
|---|---|---|
| Toolchain | nightly | **stable** |
| Frontend | Leptos 0.6 + stylers + nightly leptos | **Leptos 0.8** + plain CSS |
| Plugin registry | compile-time codegen via `linker` + `link_proc_macro` | runtime, via `[[plugin]]` entries in main `config.toml` |
| Plugin process model | one big binary, plugins linked in | **one independent Rocket process per plugin** |
| Plugin auth | shared cookie via the main server | **per-plugin bearer tokens** (main server attaches them on proxy calls) |
| Storage | shared MongoDB `events` collection | **one SQLite file per plugin** (`<data_dir>/plugins/<name>/events.db`) |
| Blob storage | base64 blobs inside Mongo rows | on-disk asset folder (`<data_dir>/plugins/<name>/assets/`) |
| Plugin frontend | server-linked Rust crates | **Leptos+trunk wasm bundles** loaded into shadow DOM |
| Experiences | `--features=experiences` flag on the main server | reseparated; experiences becomes its own plugin (port deferred — see `experiences/PORT_PLAN.md`) |

Per-plugin migration spec lives at `timeline/plugins/<name>/MIGRATION.md`
in each plugin's git repo. The general shape is the same everywhere:
nanos → millis on timing; Mongo `event` field → JSON in the SQLite
`data` column; base64 blobs decoded into the asset folder; on-disk
caches renamed and relocated under `<data_dir>/plugins/<name>/cache/`.

## Step-by-step

### 1. Pick a `data_dir`

Default is `./data` relative to the server's CWD. For systemd, use
`/var/lib/timeline` (the NixOS module does this).

```
<data_dir>/
  plugins/<name>/{events.db, assets/, cache/}
  plugin_web/<name>/{index.html, *.js, *.wasm}
```

### 2. Build the artifacts

```
cd timeline/frontend && trunk build --release
cd timeline/server   && cargo build --release
# per plugin (each a separate git repo):
cd plugins/<name>/server && cargo build --release
cd plugins/<name>/client && trunk build --release
```

The Nix flake does all of this (`nix build .#server`, `nix build .#frontend`).

### 3. Lay out plugin web bundles

For each plugin you intend to run, copy its `client/dist/*` into
`<data_dir>/plugin_web/<plugin_name>/`. The main server serves these
statically at `/plugin_web/<name>/<path..>`, and the plugin's manifest
points its `web_entry` at the JS file (e.g.
`timeline_plugin_steam_client.js`).

### 4. Write `server/config.toml`

```toml
port = 8002
password = "<your old cookie password, for backward compat>"
data_dir = "./data"
# error_report_url = "https://..."

[[plugin]]
name  = "timeline_plugin_steam"
url   = "http://127.0.0.1:9001"
token = "<random shared secret>"

[[plugin]]
name  = "timeline_plugin_spotify"
url   = "http://127.0.0.1:9003"
token = "<random shared secret>"

# … one entry per plugin you run …
```

### 5. Write each plugin's `config.toml`

Each plugin's repo ships a `config.toml.example`. The shape is:

```toml
[plugin]
name = "<must match the main server's [[plugin]].name>"
port = 9001
token = "<must match the main server's [[plugin]].token>"
# data_dir = "./data"          # default; same dir as the main server
# error_report_url = "..."     # optional

[config]
# whatever the plugin specifically needs (api_key, paths, …)
```

### 6. Migrate the data

For each plugin, follow its `MIGRATION.md`. The general shape:

- Open the old MongoDB and dump rows where `plugin == "<name>"`.
- For each row, derive the new SQLite primary key (often the same `id`),
  convert `timing[0]/1_000_000` → `start_ts`, etc.
- For plugins that stored blobs in Mongo (`event_type: "Cover"`,
  `event_type: "ImageData"`, etc.), base64-decode the blob and write it
  to `<data_dir>/plugins/<name>/assets/<derived-name>`. The event row's
  `data` JSON gets a relative reference (`cover_asset`, `image_asset`).
- Cache files (`cache/timeline_plugin_*`) get renamed/moved under
  `<data_dir>/plugins/<name>/cache/<key>.json`.

The migration runner itself is **not** in this commit — the per-plugin
specs are written so a future Claude session can generate transition
software per plugin on demand.

### 7. Run

Run each plugin's binary as a separate process (one shell each, or
systemd, or `nix run .#timeline-plugin-<name>` — see `flake.nix` for
the NixOS module that wires everything up).

```sh
# In separate terminals or under systemd:
cd plugins/timeline_plugin_steam/server && ./target/release/timeline_plugin_steam_server
cd plugins/timeline_plugin_spotify/server && ./target/release/timeline_plugin_spotify_server
# … etc …
cd timeline/server && ./target/release/server
```

### 8. Smoke

```
./tests/e2e/smoke.sh        # builds + curl probes (zero plugins config)
./tests/e2e/serve.sh        # leaves a server running for browser smoke
```

## What about `timeline_plugin_experience`?

Redundant under the new architecture. Once experiences itself is ported
(see `experiences/PORT_PLAN.md`), drop `timeline_plugin_experience`
from the main `[[plugin]]` list — the experiences plugin replaces it.
Until experiences is ported, you can keep `timeline_plugin_experience`
running, but you'll be on a hybrid setup.

## Secrets

The pre-rework `server/config.toml` had real Spotify tokens, Steam API
keys, etc. inline. Those values are still in this repo's git history
(git won't lose them on a `git rm`); rotate them at the upstream
provider before exposing this code anywhere.

The new `config.toml` is in `.gitignore`; commit only
`config.toml.example`.
