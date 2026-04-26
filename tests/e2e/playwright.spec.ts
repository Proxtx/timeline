// Browser-side smoke for the rebuilt timeline frontend.
//
// Boots the same sandbox `smoke.sh` does (build server + frontend, run
// server on :18002 with zero plugins), then drives Chromium through
// Playwright. Verifies that:
//   1. Wasm boots and the title bar renders.
//   2. Auth-required routes show the Login form when unauthenticated.
//   3. Setting the `pwd` cookie unlocks the timeline view.
//   4. The plugin manifest is fetched and the (empty) app selector renders.
//
// Run (two terminals):
//   1) `./tests/e2e/smoke.sh`                    — leaves curl probes
//                                                  passing then exits;
//                                                  re-run with the
//                                                  serve helper below
//                                                  to keep it up
//   2) `./tests/e2e/serve.sh`                    — boots server on
//                                                  :18002 and blocks
//   3) In another terminal:
//      `npx playwright test tests/e2e/playwright.spec.ts \
//          --browser=chromium`                   — runs this spec
//
// Either install Playwright's bundled chromium
// (`npx playwright install chromium`) and use --browser=chromium, or
// set PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH at a system chrome.

import { test, expect, type Page } from "@playwright/test";

const PORT = 18002;
const PASSWORD = "smoke-test-pwd";
const BASE = `http://127.0.0.1:${PORT}`;

async function setPasswordCookie(page: Page) {
  await page.context().addCookies([
    {
      name: "pwd",
      value: PASSWORD,
      url: BASE,
      sameSite: "None",
      secure: false,
    },
  ]);
}

test("wasm boots and title bar renders", async ({ page }) => {
  await page.goto(`${BASE}/timeline`);
  // Title bar's <h1>Timeline</h1>
  await expect(page.locator("h1.title")).toHaveText("Timeline", { timeout: 15_000 });
});

test("unauth user sees login form", async ({ page }) => {
  // Make sure no cookie leaks across tests.
  await page.context().clearCookies();
  await page.goto(`${BASE}/timeline`);
  await expect(page.locator("input.pwdInput")).toBeVisible({ timeout: 15_000 });
});

test("with valid cookie, timeline body renders", async ({ page }) => {
  await setPasswordCookie(page);
  await page.goto(`${BASE}/timeline`);
  // The TimelineBar should mount once auth passes.
  await expect(page.locator(".timelineBar")).toBeVisible({ timeout: 15_000 });
});

test("zero plugins → empty app selector, no crash", async ({ page }) => {
  await setPasswordCookie(page);
  await page.goto(`${BASE}/timeline`);
  // Even with zero plugins, the appSelector div is rendered.
  await expect(page.locator(".appSelector")).toBeAttached({ timeout: 15_000 });
});
