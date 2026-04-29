// Browser-side smoke for the rebuilt timeline frontend.
//
// Two test files in one — Playwright runs each `test()` independently
// against whatever stack is up at http://127.0.0.1:18002:
//
//   "baseline:*"   work against the empty-plugins stack (./tests/e2e/serve.sh)
//   "tier1:*"      work against the six-plugin stack
//                  (./tests/e2e/serve_tier1.sh)
//
// Boot one of those in a separate terminal first, then in another:
//   npx playwright test
//
// (Tests are tagged with prefixes; you can filter via --grep "tier1" or
//  --grep "baseline" if you only want one set.)

import { test, expect, type Page } from "@playwright/test";

const PORT = 18002;
const PASSWORD = "smoke-test-pwd";
const BASE = `http://127.0.0.1:${PORT}`;

const TIER1_PLUGINS = [
  "timeline_plugin_text",
  "timeline_plugin_web",
  "timeline_plugin_notification",
  "timeline_plugin_usage",
  "timeline_plugin_git",
  "timeline_plugin_media_scan",
];

async function setPasswordCookie(page: Page) {
  await page.context().addCookies([
    {
      name: "pwd",
      value: PASSWORD,
      domain: "127.0.0.1",
      path: "/",
      sameSite: "Lax",
    },
  ]);
}

// ---------------- baseline (works against empty-plugins server) ----------------

test("baseline: wasm boots and title bar renders", async ({ page }) => {
  await page.goto(`${BASE}/timeline`);
  await expect(page.locator("h1.title")).toHaveText("Timeline", { timeout: 15_000 });
});

test("baseline: unauth user sees login form", async ({ page }) => {
  await page.context().clearCookies();
  await page.goto(`${BASE}/timeline`);
  await expect(page.locator("input.pwdInput")).toBeVisible({ timeout: 15_000 });
});

test("baseline: with valid cookie, timeline body renders", async ({ page }) => {
  await setPasswordCookie(page);
  await page.goto(`${BASE}/timeline`);
  await expect(page.locator(".timelineBar")).toBeVisible({ timeout: 15_000 });
});

test("baseline: app selector rendered (may be empty)", async ({ page }) => {
  await setPasswordCookie(page);
  await page.goto(`${BASE}/timeline`);
  await expect(page.locator(".appSelector")).toBeAttached({ timeout: 15_000 });
});

// ---------------- tier-1 (requires serve_tier1.sh stack) ----------------

test("tier1: /api/plugins returns all six manifests", async ({ request }) => {
  const res = await request.post(`${BASE}/api/plugins`, {
    headers: { Cookie: `pwd=${PASSWORD}` },
  });
  expect(res.ok()).toBe(true);
  const body = await res.json();
  // Wire shape: { Ok: [ {name, ...}, ... ] }
  expect(body.Ok).toBeDefined();
  const names = body.Ok.map((m: { name: string }) => m.name).sort();
  expect(names).toEqual([...TIER1_PLUGINS].sort());
});

test("tier1: /api/events fan-out includes every plugin", async ({ request }) => {
  const day = new Date();
  day.setUTCHours(0, 0, 0, 0);
  const start = day.toISOString();
  const tomorrow = new Date(day.getTime() + 24 * 60 * 60 * 1000);
  const end = tomorrow.toISOString();
  const res = await request.post(`${BASE}/api/events`, {
    headers: { Cookie: `pwd=${PASSWORD}`, "Content-Type": "application/json" },
    data: { start, end },
  });
  expect(res.ok()).toBe(true);
  const body = await res.json();
  expect(body.Ok).toBeDefined();
  for (const plugin of TIER1_PLUGINS) {
    // Some plugins may have empty arrays if their request_loop hasn't
    // run yet; we just need the key present in the fan-out map.
    expect(body.Ok[plugin]).toBeDefined();
  }
});

test("tier1: app selector renders at least one plugin icon", async ({ page }) => {
  // The AppSelect filters to plugins that have events in the
  // current_range (default first hour of day). The probe deliberately
  // injects text/usage events in that hour; we assert the weaker
  // invariant: at least one plugin icon shows up.
  await setPasswordCookie(page);
  await page.goto(`${BASE}/timeline`);
  await expect(page.locator(".appSelector .iconWrap").first()).toBeAttached({
    timeout: 20_000,
  });
});

test("tier1: clicking a plugin shows its events", async ({ page }) => {
  await setPasswordCookie(page);
  await page.goto(`${BASE}/timeline`);
  await expect(page.locator(".appSelector .iconWrap").first()).toBeAttached({
    timeout: 20_000,
  });
  await page.locator(".appSelector .iconWrap").first().click();
  await expect(page.locator(".eventRow").first()).toBeAttached({ timeout: 15_000 });
});

test("tier1: text-plugin event surfaces via /event/latest", async ({ page }) => {
  // /event/latest fetches events from the last hour. The probe
  // injects a text event anchored at "now"; the test clicks each
  // plugin icon in turn and inspects the event row's shadow root
  // for the "smoke test" body that timeline_plugin_text renders.
  page.on("console", (m) => console.log(`[browser ${m.type()}]`, m.text()));
  page.on("pageerror", (e) => console.log("[browser pageerror]", e.message));
  await setPasswordCookie(page);
  await page.goto(`${BASE}/event/latest`);

  await expect(page.locator(".appSelector .iconWrap").first()).toBeAttached({
    timeout: 20_000,
  });

  const icons = page.locator(".appSelector .iconWrap");
  const count = await icons.count();
  console.log(`found ${count} iconWraps`);
  let matched = false;
  for (let i = 0; i < count && !matched; i++) {
    await icons.nth(i).click();
    // Wait briefly for event rows to mount; if none, move on.
    try {
      await page.locator(".eventRow .eventHeader").first().waitFor({
        state: "attached",
        timeout: 3_000,
      });
    } catch {
      continue;
    }
    const headers = page.locator(".eventRow .eventHeader");
    const total = await headers.count();
    for (let j = 0; j < total && !matched; j++) {
      await headers.nth(j).click();
      try {
        await page.waitForFunction(
          () => {
            for (const h of document.querySelectorAll(".pluginHost")) {
              const sr = (h as HTMLElement).shadowRoot;
              if (!sr) continue;
              const el = sr.querySelector(".text-body");
              if (el && el.textContent?.includes("smoke test")) return true;
            }
            return false;
          },
          null,
          { timeout: 3_000 }
        );
        matched = true;
      } catch {
        // not on this row; collapse and try next
      }
    }
  }
  expect(matched).toBeTruthy();
});
