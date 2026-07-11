import { expect, test, type Page } from '@playwright/test';

const installTauriMock = async (page: Page, theme: 'light' | 'dark') => {
  await page.addInitScript((selectedTheme) => {
    localStorage.setItem('bdl.theme', selectedTheme);
    let callbackId = 1;
    Object.assign(window, {
      __TAURI_INTERNALS__: {
        metadata: { currentWindow: { label: 'main' } },
        transformCallback: () => callbackId++,
        unregisterCallback: () => undefined,
        convertFileSrc: (path: string) => path,
        invoke: async (command: string) => {
          if (command === 'account_get') {
            return { logged_in: false, name: null, avatar_url: null, mid: null, vip_label: null };
          }
          if (command === 'queue_list') return [];
          if (command === 'queue_startup_recovery') {
            return { task_ids: [], auto_recovery_enabled: false };
          }
          if (command === 'plugin:event|listen') return callbackId++;
          return null;
        },
      },
      __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener: () => undefined },
    });
  }, theme);
};

for (const theme of ['light', 'dark'] as const) {
  for (const viewport of [
    { name: 'standard', width: 1280, height: 800 },
    { name: 'narrow', width: 900, height: 700 },
  ]) {
    test(`${theme} ${viewport.name} parse workspace`, async ({ page }) => {
      await page.setViewportSize(viewport);
      await installTauriMock(page, theme);
      await page.goto('/');
      await expect(page.getByRole('heading', { name: '从链接整理下载内容' })).toBeVisible();
      await expect(page).toHaveScreenshot(`parse-${theme}-${viewport.name}.png`, {
        animations: 'disabled',
        caret: 'hide',
        maxDiffPixelRatio: 0.01,
      });
    });
  }
}

test('native browser context menu is suppressed', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await expect(page.getByRole('heading', { name: '从链接整理下载内容' })).toBeVisible();

  const contextMenuAllowed = await page.evaluate(() =>
    document.body.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, cancelable: true })),
  );

  expect(contextMenuAllowed).toBe(false);
});
