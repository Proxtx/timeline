# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repository is

Self-hosted life-logging server. The server is a Rocket app on **stable Rust**; the frontend is a **Leptos 0.8 CSR** wasm bundle built with trunk; data lands in **per-plugin SQLite** files plus on-disk asset folders. Plugins are **standalone HTTP services** that the main server proxies, each one independently buildable and runnable. The pre-rework MongoDB / nightly-rust / compile-time-codegen setup is gone; see `MIGRATION.md` if you're coming from that.

## Repository layout

- `server/` — main Rocket binary. Loads `config.toml`, registers plugins from `[[plugin]]` tables, proxies `/api/plugin/<name>/<path..>` to each plugin, fan-outs `/api/events` and `/api/markers`, aggregates `/api/plugins` manifest.
- `frontend/` — Leptos 0.8 CSR app. Routes: `/timeline[/:date]`, `/event/latest[/exclude/:exclude]`. Loads each plugin's wasm bundle dynamically and mounts it into a shadow root per event card.
- `types/` — `CompressedEvent`, `Timing`, `TimeRange`, `APIError`, `APIResult`, `Marker`. No feature flags; no MongoDB.
- `timeline_plugin_sdk/` — server-side helper crate plugin authors depend on. Provides the `Plugin` trait, SQLite event store (`Db`), `AssetStore`, `Cache`, `ErrorReporter`, bearer-token guard, `launch::<P>()` entrypoint.
- `timeline_plugin_client_sdk/` — wasm-side helper crate. `plugin_entry!` declarative macro, `mount_plugin` (open shadow root + `leptos::mount::mount_to`), `ApiClient`, `Style`, `PluginContext`.
- `plugins/<name>/` — independent git repos for each plugin. Each has `server/` (Rocket binary on `timeline_plugin_sdk`), `client/` (Leptos+trunk wasm bundle on `timeline_plugin_client_sdk`), `MIGRATION.md`, `config.toml.example`. Note `plugins/` itself is gitignored from the timeline repo — each plugin has its own `.git/`.
- `flake.nix` — `packages.server`, `packages.frontend`, `devShells.default` (rust toolchain + trunk + sqlite), `nixosModules.default` with `services.timeline` running the main server + one systemd unit per plugin.
- `tests/e2e/` — `smoke.sh` (curl-based server probes; passes), `serve.sh` (host the server for browser testing), `playwright.spec.ts` (browser smoke; runner not wired in this session — see `tests/e2e/README.md`).

## How a plugin works

Each plugin is a standalone process with two pieces:

1. A Rocket binary (`plugins/<name>/server/`) that implements `timeline_plugin_sdk::Plugin`:
   - `Plugin::events(range)` — return `Vec<CompressedEvent>` overlapping the range. Most plugins read from `self.ctx.db.query_range_typed::<MyPayload>(&range)`. Plugins that synthesize events on demand (location, usage) skip the DB entirely.
   - `Plugin::manifest()` — returns the manifest the main server aggregates: `name`, `display_name`, `style`, `icon?`, `web_entry?`.
   - `Plugin::request_loop()` — optional; the SDK reschedules it after the returned `Duration`. Panics are caught and reported via `ErrorReporter`.
   - `Plugin::routes()` — optional; plugin-specific Rocket routes get mounted alongside the SDK's standard ones.
   - `Plugin::rocket_attach(rocket)` — optional; for plugins that need their own Rocket state (e.g. an OG cache, an RSA verifying key).
   - The SDK's standard routes — `POST /events`, `GET /manifest`, `GET /assets/<path..>`, `GET /health` — are always present.

2. A Leptos+trunk client (`plugins/<name>/client/`) that exports a `__timeline_plugin_render` symbol via `plugin_entry!(render)`. The main frontend dynamic-imports the client's JS, mounts it into a shadow root per event card, and hands it a `PluginContext { plugin_name, api_base, event, style, mode }`.

Communication: main server → plugin uses `Authorization: Bearer <token>` from the plugin's entry in the main `config.toml`. Plugins that need URL-time auth for external clients (notification, unify) layer that on top.

## Common commands

Top of repo:
- `./tests/e2e/smoke.sh` — full build + curl probes. `--skip-build` to skip `trunk build` / `cargo build`.
- `./tests/e2e/serve.sh` — boot the server for manual browser testing (port 18002).

Per-crate:
- Frontend release: `cd frontend && trunk build --release`
- Frontend dev watch: `cd frontend && trunk serve` (or `cargo watch -i dist -- trunk build`)
- Server: `cd server && cargo +stable run --release`
- SDK type-check: `cd timeline_plugin_sdk && cargo +stable check`
- Client SDK type-check: `cd timeline_plugin_client_sdk && cargo +stable check --target wasm32-unknown-unknown`
- A specific plugin server: `cd plugins/<name>/server && cargo +stable run`
- A specific plugin client: `cd plugins/<name>/client && trunk build --release`

The `rust-toolchain.toml` at the timeline root pins **stable** with the wasm32 target, so `cargo` (no `+stable`) does the right thing in the timeline workspace. Plugin sub-repos don't share that toolchain file by default; use `cargo +stable` if you've still got a nightly default.

## Configuration

`server/config.toml.example` shows the new shape. Local `config.toml` is gitignored. Per-plugin entries:

```toml
[[plugin]]
name = "timeline_plugin_steam"
url  = "http://127.0.0.1:9001"
token = "shared-with-the-plugin"
```

Each plugin's own `config.toml` follows the SDK shape: `[plugin]` (port, token, optional data_dir / display_name / error_report_url) plus `[config]` (whatever the plugin's own deserialized struct expects).

## Routing model

- Main server's `/` → `FileServer` from `../frontend/dist/`.
- `/api/auth` cookie-checks the main `pwd` cookie against the main config password.
- `/api/events`, `/api/markers`, `/api/plugins` cookie-auth then fan-out to plugins over HTTP with bearer tokens.
- `/api/plugin/<name>/<path..>` proxies any GET / POST / PUT / DELETE to the plugin's base URL with bearer added (the proxy itself doesn't cookie-check; plugins implement their own URL-time auth where needed — e.g. signed file URLs in media_scan / documents, or per-plugin password fields in notification / unify).
- `/plugin_web/<name>/<path..>` serves trunk dist outputs from `<data_dir>/plugin_web/<name>/`. The frontend dynamic-imports JS from here.
- 404 catcher returns `dist/index.html` for SPA routing.

## Per-plugin storage layout

```
<data_dir>/
  plugins/
    <plugin_name>/
      events.db                          SQLite (id TEXT PRIMARY KEY,
                                          start_ts/end_ts INTEGER ms,
                                          title TEXT, data TEXT JSON)
      assets/<rel>                       blobs lifted out of events
      cache/<key>.json                   plugin-side state cache
      signing_key.pem                    media_scan / documents
  plugin_web/<plugin_name>/<trunk dist>
```

## Gotchas

- SQLite times are **milliseconds** (`timestamp_millis()`); the old Mongo-era code stored nanoseconds. Per-plugin `MIGRATION.md` files spell out the `/1_000_000` conversion.
- Trunk filehash is disabled per-plugin via `Trunk.toml` (`filehash = false`) so the manifest's `web_entry` filename doesn't change per build.
- Plugin `cargo check` inside a sub-repo can produce a stale `Cargo.lock` if you switch SDK deps — `rm Cargo.lock && cargo check` if you see weird `leptos`/`leptos_macro` version-mismatch errors.
- `cargo tree -p <pkg> --target wasm32-unknown-unknown` has been observed to hang for >1h on this project; prefer `grep '^name = "<pkg>"' Cargo.lock -A1` or build logs for inspecting versions.
- Experiences rework is **deferred**; see `experiences/PORT_PLAN.md`. The old `timeline_plugin_experience` plugin is redundant once experiences ships; drop it from the main server's `[[plugin]]` list when running the new experiences plugin.
