import { defineConfig, devices } from "@playwright/test";

// Locate the chromium binary that `npx playwright install chromium` dropped
// under ~/.cache/ms-playwright/. The MCP plugin in this dev environment
// expects /opt/google/chrome/chrome (Google Chrome channel) which we don't
// have, so we explicitly pin the bundled-chromium binary here.
const home = process.env.HOME ?? "";
const candidates = [
  `${home}/.cache/ms-playwright/chromium-1217/chrome-linux64/chrome`,
  `${home}/.cache/ms-playwright/chromium_headless_shell-1217/chrome-headless-shell-linux64/chrome-headless-shell`,
];
import { existsSync } from "node:fs";
const executablePath = candidates.find((p) => existsSync(p));

export default defineConfig({
  testDir: ".",
  testMatch: "*.spec.ts",
  fullyParallel: false,
  workers: 1,
  reporter: [["list"], ["html", { outputFolder: "_run/playwright-report", open: "never" }]],
  outputDir: "_run/playwright-output",
  use: {
    baseURL: "http://127.0.0.1:18002",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: {
        ...devices["Desktop Chrome"],
        launchOptions: executablePath ? { executablePath } : {},
      },
    },
  ],
});
