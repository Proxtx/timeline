# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Layout

This directory contains **two sibling Rust projects** that are developed together but live in separate git repositories:

- `timeline/` — a self-hosted life-logging server + Leptos CSR frontend. Plugins in `timeline/plugins/*` each contain `server/` and `client/` crates.
- `experiences/` — optional companion project that organizes timeline events into user-curated "experiences". Only relevant when the `experiences` feature is enabled on timeline.

They reference each other by path through two plain-text pointer files:

- `timeline/experiences_location.txt` → relative path to the experiences repo.
- `experiences/timeline_location.txt` → relative path to the timeline repo.

Both files must exist and be correct for a cross-project build. They are read by the `linker` binaries (see below), not by cargo directly.

## The plugin + linker system (most important to understand)

There is **no static `Cargo.toml`** enumerating plugins. Instead, each project has a `linker/` binary that regenerates `link/Cargo.toml` by scanning `plugins/` at build-time. The generated `dyn_link` (timeline) / `link` (experiences) crate then feature-gates every discovered plugin.

### Timeline flow

1. `timeline/linker/src/main.rs` reads `timeline/plugins/*`, reads `../experiences_location.txt`, and writes `timeline/link/Cargo.toml` with one optional `*_server` / `*_client` dependency per plugin, plus matching entries on the `server` / `client` feature lists.
2. `link_proc_macro::generate_server_plugins` / `generate_client_plugins` (expanded in `timeline/link/src/*_plugins.rs`) produce the runtime `Plugins` struct that actually instantiates every compiled-in plugin.
3. `link_proc_macro::generate_available_plugins` (in `timeline/types/src/available_plugins.rs`) extends the `AvailablePlugins` enum with one variant per plugin.
4. The plugin list is sourced from `./plugins.txt` if present (this is how the Nix flake pins plugins), otherwise by reading `../plugins/` directly. `timeline/types/build.rs` has a second, independent codegen path that writes a plugins.rs to `OUT_DIR` — both must stay in sync with the plugin directory layout.

### Run sequence

From `timeline/`:
```
cd linker && cargo run             # regenerate link/Cargo.toml (pass "disable" to omit frontend deps)
cd ../frontend && trunk build --release
cd ../server && cargo run --release
```

Running `cargo run` / `cargo build` inside `server/` or `frontend/` **before** running `linker` will fail or produce stale output — the linker is the source of truth for which plugins are wired in.

### Experiences flow

Same pattern, but the experiences `linker` generates **two** files:
- `experiences/timeline_types/Cargo.toml` (points the shared types crate at `timeline/types` with the `experiences,client` features)
- `experiences/link/Cargo.toml` (the plugin feature matrix)

To enable experiences inside timeline: build with `--features=experiences` on `server` (and on `frontend` via trunk), ensure `experiences_location.txt` is correct, and set `experiences_url` in `server/config.toml`.

## Common commands

Timeline:
- Regenerate plugin wiring: `cd timeline/linker && cargo run` (append `disable` to skip frontend/client plugins, used by the Nix build).
- Frontend release build: `cd timeline/frontend && trunk build --release`
- Frontend dev watch: `cd timeline/frontend && cargo watch -i dist -- trunk build` (see `frontend/dev_run.txt`).
- Server run: `cd timeline/server && cargo run --release` (serves the `frontend/dist/` directory and mounts plugin routes under `/api/plugin/{plugin}`).
- With experiences: add `--features=experiences` to the server build and set the frontend's `experiences` feature.

Experiences:
- Regenerate wiring: `cd experiences/linker && cargo run`
- Frontend watch: `cd experiences/frontend && cargo watch -i dist -- trunk build` (see `frontend/run.txt`).

Toolchain: Rust **nightly** is required (`timeline/server/Cargo.toml` pins `channel = "nightly"`; the frontend uses `#![feature(let_chains)]` and leptos nightly). The Nix flake at `timeline/flake.nix` uses `rust-bin.nightly.latest.default` via crane and is the canonical production build.

## Runtime architecture (server)

- `timeline/server/src/main.rs` boots Rocket, loads `config.toml`, opens a MongoDB connection (`server_api::db::Database`), then calls `Plugins::init` (the codegen'd `dyn_link::server_plugins::Plugins`) which constructs every compiled-in plugin with a `PluginData { database, config, plugin, error_url }`.
- Each plugin implements `server_api::plugin::PluginTrait`: `get_compressed_events` is mandatory; `request_loop` / `request_loop_mut` are optional async loops the `PluginManager` reschedules on their returned `Duration` (see `server/src/plugin_manager.rs`). Panics in plugin loops are caught, reported via `error_url`, and the loop sleeps 300s before retrying.
- The built-in `error` plugin is inserted manually after codegen because it lives in `server_api::error`, not in `plugins/`.
- Routes: `FileServer` serves `../frontend/dist/`, core API lives at `/api/*`, and each plugin's routes mount at `/api/plugin/{plugin}`. The 404 catcher falls back to `dist/index.html` for SPA routing.

## Frontend architecture

- `timeline/frontend` is a Leptos CSR app (`trunk` + `stylers` for scoped CSS). `build.rs` writes the stylers output to `target/generated.css`.
- Routes: `/timeline[/:date]` and `/event/latest[/exclude/:exclude]` (see `main.rs`).
- `plugin_manager::PluginManager` (client side) mirrors the server's plugin registry; each plugin's `client/` crate supplies UI.
- Auth is gated at `api_request::<(), ()>("/auth", &())`; failures render the `Login` wrapper.

## Config

`timeline/server/config.toml` holds DB connection, port, password, optional `experiences_url`, and per-plugin `[plugin_config.*]` tables. The `password` and `db_connection_string` in the checked-in file are real dev values — do not commit new secrets here and rotate any that leak.

## Gotchas

- The plugin enum (`AvailablePlugins`) is generated in **two** places (the proc macro in `types/src/available_plugins.rs` and `types/build.rs`). Both read `plugins/` from the CWD at compile time; running cargo from an unexpected directory can produce a mismatched enum and confusing type errors.
- The linker writes into tracked files (`link/Cargo.toml`, `timeline_types/Cargo.toml`). Diff noise from those files after a build is expected; don't commit it unless you're intentionally changing the plugin set shipped in git.
- `experiences_navigator` is referenced in `timeline/link/Cargo.toml` with a hard-coded absolute path outside this repo tree. The linker rewrites this path to match `experiences_location.txt` — re-run the linker after moving the experiences checkout.
