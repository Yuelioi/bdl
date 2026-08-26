import { expect, test, type Page } from '@playwright/test';

const installTauriMock = async (page: Page, theme: 'light' | 'dark', environmentReady = true) => {
  await page.addInitScript(({ selectedTheme, environmentReady }) => {
    localStorage.setItem('bdl.theme', selectedTheme);
    let callbackId = 1;
    const invocations: string[] = [];
    Object.assign(window, {
      __BDL_TEST_INVOKES__: invocations,
      __TAURI_INTERNALS__: {
        metadata: { currentWindow: { label: 'main' } },
        transformCallback: () => callbackId++,
        unregisterCallback: () => undefined,
        convertFileSrc: (path: string) => path,
        invoke: async (command: string) => {
          invocations.push(command);
          if (command === 'account_get') {
            return { logged_in: false, name: null, avatar_url: null, mid: null, vip_label: null };
          }
          if (command === 'settings_get') {
            return {
              settings_schema_version: 1,
              download_dir: 'C:\\Downloads',
              naming_template: '{title}/P{part_index} - {part_title}.{ext}',
              quality: 'best',
              archive_mode: 'fast',
              archive_assets: { cover: true, subtitles: true, danmaku: true, nfo: true },
              output_extension: 'mp4',
              duplicate_naming_strategy: 'append_suffix',
              audio_quality: 'best',
              codec: 'auto',
              missing_quality_policy: 'lower',
              ffmpeg_path: null,
              retain_raw_streams: false,
              embed_cover: false,
              embed_subtitles: false,
              proxy_url: null,
              log_level: 'info',
              data_dir: null,
              concurrent_tasks: 1,
              retry_count: 3,
              segment_count: 4,
              global_speed_limit_bytes_per_second: null,
              startup_auto_recovery: false,
              auto_refresh_expired_urls: true,
            };
          }
          if (command === 'environment_health') {
            return {
              ready: environmentReady,
              download_directory: {
                status: 'ready',
                path: 'C:\\Downloads',
                message: '保存目录可写。',
              },
              ffmpeg: environmentReady
                ? {
                    status: 'ready',
                    source: 'system',
                    path: 'C:\\ffmpeg.exe',
                    version: 'ffmpeg version test',
                    message: 'FFmpeg 可用。',
                  }
                : {
                    status: 'missing',
                    source: 'system',
                    path: 'ffmpeg',
                    version: null,
                    message: '未找到可用的 FFmpeg。',
                  },
            };
          }
          if (command === 'parse_create_source') {
            return {
              source: {
                id: 'video:fixture',
                kind: 'video',
                input: 'BV1xx411c7mD',
                title: '测试视频',
                loaded_count: 1,
                total_count: 1,
                has_more: false,
              },
              groups: [
                {
                  id: 'group:video:fixture',
                  kind: 'video',
                  title: '测试视频',
                  page: null,
                  items: [
                    {
                      id: 'item:video:fixture',
                      title: '测试视频',
                      owner_name: '测试用户',
                      owner_mid: '42',
                      cover_url: null,
                      duration_seconds: 60,
                      parts: [
                        {
                          id: 'part:video:fixture',
                          title: '测试视频',
                          aid: 1,
                          bvid: 'BV1xx411c7mD',
                          cid: 2,
                          duration_seconds: 60,
                          streams: [],
                          assets: [],
                        },
                      ],
                    },
                  ],
                },
              ],
            };
          }
          if (command === 'account_login_qr_start') {
            return {
              qr_url: 'https://example.test/login',
              qrcode_key: 'fixture-key',
              qr_image_svg:
                '<svg xmlns="http://www.w3.org/2000/svg" width="180" height="180"><rect width="180" height="180" fill="white"/><rect x="20" y="20" width="140" height="140" fill="black"/></svg>',
              expires_in_seconds: 180,
            };
          }
          if (command === 'account_login_qr_poll') {
            return { status: 'waiting', message: '等待扫码', account: null };
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
  }, { selectedTheme: theme, environmentReady });
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
      await expect(page.getByRole('button', { name: '开始解析' })).toBeEnabled();
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

test('the QR placeholder starts a session without a duplicate footer action', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await expect(page.getByRole('heading', { name: '从链接整理下载内容' })).toBeVisible();

  await page.getByRole('button', { name: '登录' }).click();
  await page.getByRole('menuitem', { name: '登录' }).click();

  const dialog = page.getByRole('dialog', { name: '登录' });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole('button', { name: '获取登录二维码' })).toBeVisible();
  await expect(dialog.getByRole('button', { name: '开始扫码' })).toHaveCount(0);

  await dialog.getByRole('button', { name: '获取登录二维码' }).click();

  await expect(dialog.locator('.qr-box img')).toBeVisible();
  await expect(dialog.getByText('等待扫码')).toBeVisible();
  await expect(dialog.getByRole('button', { name: '刷新二维码' })).toBeVisible();
});

test('startup checks the environment once and blocks parsing when FFmpeg is missing', async ({ page }) => {
  await installTauriMock(page, 'light', false);
  await page.goto('/');

  const dialog = page.getByRole('dialog', { name: '下载环境未就绪' });
  await expect(dialog).toBeVisible();
  await dialog.getByRole('button', { name: '稍后处理' }).click();

  await expect(page.getByRole('button', { name: '开始解析' })).toBeDisabled();
  await expect(page.getByText('下载环境未就绪，修复保存目录或 FFmpeg 后才能解析。')).toBeVisible();

  const environmentChecks = await page.evaluate(
    () => ((window as unknown as { __BDL_TEST_INVOKES__: string[] }).__BDL_TEST_INVOKES__ ?? []).filter(
      (command) => command === 'environment_health',
    ).length,
  );
  expect(environmentChecks).toBe(1);
});

test('opening download settings reuses startup environment health', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await expect(page.getByRole('button', { name: '开始解析' })).toBeEnabled();

  await page.getByLabel('链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();
  await page.getByRole('treeitem', { name: /测试视频/ }).click();
  await page.getByRole('button', { name: '下载所选 (1)' }).click();

  await expect(page.getByRole('dialog', { name: '下载设置' })).toBeVisible();
  const environmentChecks = await page.evaluate(
    () => ((window as unknown as { __BDL_TEST_INVOKES__: string[] }).__BDL_TEST_INVOKES__ ?? []).filter(
      (command) => command === 'environment_health',
    ).length,
  );
  expect(environmentChecks).toBe(1);
});
