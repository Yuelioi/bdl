import { defineConfig } from '@playwright/test';

const productionPreview = process.env.BDL_BROWSER_PRODUCTION === '1';

export default defineConfig({
  // Functional browser checks; no screenshot baselines or visual acceptance.
  testDir: './tests/browser',
  fullyParallel: false,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? 'github' : 'list',
  use: {
    baseURL: 'http://127.0.0.1:4173',
    screenshot: 'only-on-failure',
    trace: 'retain-on-failure',
    locale: 'zh-CN',
    timezoneId: 'Asia/Shanghai',
    colorScheme: 'light',
    reducedMotion: 'reduce',
  },
  webServer: {
    command: productionPreview
      ? 'pnpm exec vite preview --host 127.0.0.1 --port 4173 --strictPort'
      : 'pnpm dev --host 127.0.0.1 --port 4173',
    url: 'http://127.0.0.1:4173',
    reuseExistingServer: !process.env.CI && !productionPreview,
    timeout: 120_000,
  },
});
