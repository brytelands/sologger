import { defineConfig, devices } from '@playwright/test';

const PORT = 5199;
// Lets a machine use an already-installed Chromium instead of `npx playwright install`
const executablePath = process.env.PLAYWRIGHT_CHROMIUM_PATH || undefined;

export default defineConfig({
  testDir: './e2e',
  forbidOnly: !!process.env.CI,
  reporter: 'list',
  timeout: 45_000,
  use: {
    baseURL: `http://localhost:${PORT}/sologger/`,
    screenshot: 'only-on-failure',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'], launchOptions: { executablePath } } },
  ],
  webServer: {
    command: `npx vite --port ${PORT} --strictPort`,
    url: `http://localhost:${PORT}/sologger/`,
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
});
