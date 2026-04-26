# End-to-end smoke

Two test surfaces:

## `smoke.sh` — server-side curl probes

Builds the frontend (trunk) + server (cargo --release), boots the
server on port 18002 with a sandbox `_run/` data dir, and probes the
core API endpoints:

- `POST /api/auth` (without and with `pwd=…` cookie)
- `POST /api/events` with zero plugins → `{}`
- `POST /api/plugins` with zero plugins → `[]`
- SPA fallback for `/timeline/` and a deep link
- Static `/index.html` served from `frontend/dist/`

```sh
./tests/e2e/smoke.sh           # full build + run
./tests/e2e/smoke.sh --skip-build  # reuse existing release artifacts
```

Cleans up the spawned server on exit. All probes must pass; non-zero
exit on any failure.

## `playwright.spec.ts` — browser smoke (deferred)

Loads the served frontend in a headless browser and verifies the wasm
boots, the title bar renders, and a route navigation works. Authored
against `@playwright/test`, but **not** wired into a CI runner here —
the harness this project uses for browser automation has a stale Chrome
expectation that we haven't sorted out yet (it expects
`/opt/google/chrome/chrome` and the host doesn't have a system Chrome).

Two paths to run it:

1. **Use Playwright's bundled chromium.** `npx playwright install chromium`
   was already run by the rework session; the binary lives under
   `~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome`. Pass
   `--browser=chromium` to the playwright runner instead of letting it
   default to the Chrome channel.
2. **Point at a flatpak/distro chrome via env.** Set
   `PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH` to your Chrome wrapper.

The spec itself is straightforward enough that you can adapt to either
path without code changes; only the runner invocation differs.

The boot procedure for the browser test mirrors `smoke.sh`: same
sandbox, same build commands. See the spec file for details.
