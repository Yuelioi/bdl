import { expect, test, type Page } from '@playwright/test';

const installTauriMock = async (
  page: Page,
  theme: 'light' | 'dark',
  environmentReady = true,
  accountLoggedIn = false,
  taskCreationFails = false,
  collectionLoadDelayMs = 0,
) => {
  await page.addInitScript(
    ({ selectedTheme, environmentReady, accountLoggedIn, taskCreationFails, collectionLoadDelayMs }) => {
      localStorage.setItem('bdl.theme', selectedTheme);
      let callbackId = 1;
      let pagedFixtureLoaded = 9;
      const pagedFixtureItems = () =>
        Array.from({ length: Math.min(pagedFixtureLoaded, 9) }, (_, index) => ({
          id: `item:favorite:fixture:${index + 1}`,
          title: `分页视频 ${index + 1}`,
          owner_name: '测试用户',
          owner_mid: '42',
          cover_url: null,
          duration_seconds: 60 + index,
          parts: [
            {
              id: `part:favorite:fixture:${index + 1}`,
              title: `分页视频 ${index + 1}`,
              aid: index + 1,
              bvid: `BV1xx411c8m${index}`,
              cid: null,
              duration_seconds: 60 + index,
              streams: [],
              assets: [],
            },
          ],
        }));
      const invocations: string[] = [];
      Object.assign(window, {
        __BDL_TEST_INVOKES__: invocations,
        __TAURI_INTERNALS__: {
          metadata: { currentWindow: { label: 'main' } },
          transformCallback: () => callbackId++,
          unregisterCallback: () => undefined,
          convertFileSrc: (path: string) => path,
          invoke: async (
            command: string,
            args?: { request?: { kind?: string; input?: string; source_id?: string; limit?: number } },
          ) => {
            invocations.push(command);
            if (command === 'account_get') {
              return accountLoggedIn
                ? { logged_in: true, name: '测试用户', avatar_url: null, mid: '42', vip_label: null }
                : { logged_in: false, name: null, avatar_url: null, mid: null, vip_label: null };
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
                duplicate_naming_strategy: 'skip_existing',
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
              if (args?.request?.input === 'favorite:fixture') {
                return {
                  source: {
                    id: 'favorite:fixture',
                    kind: 'favorite',
                    input: 'favorite:fixture',
                    title: '分页收藏夹',
                    loaded_count: pagedFixtureLoaded,
                    total_count: 401,
                    has_more: true,
                  },
                  groups: [
                    {
                      id: 'group:favorite:fixture',
                      kind: 'favorite',
                      title: '分页收藏夹',
                      page: {
                        page_number: 1,
                        page_size: 20,
                        loaded_count: pagedFixtureLoaded,
                        total_count: 401,
                        has_more: true,
                      },
                      items: pagedFixtureItems(),
                    },
                  ],
                };
              }
              const isCollectedFolder = args?.request?.input?.includes('/lists/8805852') === true;
              if (isCollectedFolder) {
                if (collectionLoadDelayMs > 0) {
                  await new Promise((resolve) => window.setTimeout(resolve, collectionLoadDelayMs));
                }
                const items = Array.from({ length: 5 }, (_, index) => ({
                  id: `item:collection:fixture:${index + 1}`,
                  title: `合集视频 ${index + 1}`,
                  owner_name: null,
                  owner_mid: '42',
                  cover_url: null,
                  duration_seconds: 60,
                  parts: [
                    {
                      id: `part:collection:fixture:${index + 1}`,
                      title: `合集视频 ${index + 1}`,
                      aid: index + 1,
                      bvid: `BV1xx411c7m${index}`,
                      cid: null,
                      duration_seconds: 60,
                      streams: [],
                      assets: [],
                    },
                  ],
                }));
                return {
                  source: {
                    id: 'collection:179632001:8805852',
                    kind: 'collection',
                    input: args?.request?.input ?? '',
                    title: '双赢之路',
                    loaded_count: 5,
                    total_count: 5,
                    has_more: false,
                  },
                  groups: [
                    {
                      id: 'group:collection:179632001:8805852',
                      kind: 'collection',
                      title: '双赢之路',
                      page: { page_number: 1, page_size: 20, loaded_count: 5, total_count: 5, has_more: false },
                      items,
                    },
                  ],
                };
              }
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
            if (command === 'parse_load_all' && args?.request?.source_id === 'favorite:fixture') {
              pagedFixtureLoaded = Math.min(args.request.limit ?? 401, 401);
              const hasMore = pagedFixtureLoaded < 401;
              return {
                source: {
                  id: 'favorite:fixture',
                  kind: 'favorite',
                  input: 'favorite:fixture',
                  title: '分页收藏夹',
                  loaded_count: pagedFixtureLoaded,
                  total_count: 401,
                  has_more: hasMore,
                },
                groups: [
                  {
                    id: 'group:favorite:fixture',
                    kind: 'favorite',
                    title: '分页收藏夹',
                    page: {
                      page_number: Math.ceil(pagedFixtureLoaded / 20),
                      page_size: 20,
                      loaded_count: pagedFixtureLoaded,
                      total_count: 401,
                      has_more: hasMore,
                    },
                    items: pagedFixtureItems(),
                  },
                ],
              };
            }
            if (command === 'account_library_list') {
              if (args?.request?.kind === 'collected_favorite') {
                return {
                  items: [
                    {
                      kind: 'collected_favorite',
                      media_id: '8805852',
                      title: '双赢之路',
                      description: '测试订阅合集',
                      cover_url:
                        'data:image/svg+xml,%3Csvg xmlns="http://www.w3.org/2000/svg" width="160" height="96"%3E%3Crect width="160" height="96" fill="%23f1f1f1"/%3E%3C/svg%3E',
                      owner_name: '合集作者',
                      owner_mid: '179632001',
                      media_count: 5,
                      source_url: 'https://space.bilibili.com/179632001/lists/8805852?type=season',
                    },
                  ],
                  total: 1,
                  page: 1,
                  page_size: 20,
                  has_more: false,
                };
              }
              return {
                items: [
                  {
                    kind: 'created_favorite',
                    media_id: '69',
                    title: 'Web',
                    description: '我的收藏夹',
                    cover_url: null,
                    owner_name: '测试用户',
                    owner_mid: '42',
                    media_count: 69,
                    source_url: 'https://space.bilibili.com/42/favlist?fid=69&ftype=create',
                  },
                ],
                total: 1,
                page: 1,
                page_size: 20,
                has_more: false,
              };
            }
            if (command === 'selection_create_tasks') {
              if (taskCreationFails) {
                throw {
                  code: 'bilibili_request_rejected',
                  message:
                    'Bilibili 暂时拒绝了请求（HTTP 412），可能是访问过于频繁或触发风控。请稍后重试；持续出现时请重新登录。',
                };
              }
              return { created: [], duplicates: [], skipped_existing: 0, requires_confirmation: false };
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
    },
    { selectedTheme: theme, environmentReady, accountLoggedIn, taskCreationFails, collectionLoadDelayMs },
  );
};

test('renders navigation icon bodies before the first painted frame', async ({ page }) => {
  await page.addInitScript(() => {
    const samples: Array<{ navItems: number; icons: number }> = [];
    Object.assign(window, { __BDL_ICON_SAMPLES__: samples });
    const sample = () => {
      const navItems = document.querySelectorAll('.nav-item').length;
      if (navItems > 0) {
        const icons = Array.from(document.querySelectorAll('.nav-item svg')).filter((svg) => svg.childElementCount > 0).length;
        samples.push({ navItems, icons });
      }
      if (samples.length < 10) requestAnimationFrame(sample);
    };
    requestAnimationFrame(sample);
  });
  await installTauriMock(page, 'light');
  await page.goto('/');

  await expect(page.locator('.nav-item')).toHaveCount(5);
  await expect(page.locator('.nav-item svg')).toHaveCount(5);
  const samples = await page.evaluate(
    () => (window as Window & { __BDL_ICON_SAMPLES__?: Array<{ navItems: number; icons: number }> }).__BDL_ICON_SAMPLES__ ?? [],
  );
  expect(samples.length).toBeGreaterThan(0);
  expect(samples.some(({ navItems, icons }) => icons < navItems)).toBe(false);
});

test('renders dynamic icon buttons before paint without Iconify network access', async ({ page }) => {
  await page.addInitScript(() => {
    const samples: Array<{ buttons: number; icons: number }> = [];
    Object.assign(window, { __BDL_ICON_BUTTON_SAMPLES__: samples });
    const sample = () => {
      const buttons = document.querySelectorAll('.ui-icon-button').length;
      if (buttons > 0) {
        const icons = Array.from(document.querySelectorAll('.ui-icon-button svg')).filter((svg) => svg.childElementCount > 0).length;
        samples.push({ buttons, icons });
      }
      if (samples.length < 10) requestAnimationFrame(sample);
    };
    requestAnimationFrame(sample);
  });
  await page.route(/https:\/\/(?:api\.iconify\.design|api\.simplesvg\.com|api\.unisvg\.com)\/.*/, (route) =>
    route.abort(),
  );
  await installTauriMock(page, 'light');
  await page.goto('/');

  const appearanceButton = page.getByRole('button', { name: /外观：/ });
  await expect(appearanceButton).toBeVisible();
  await expect(appearanceButton.locator('svg')).toHaveCount(1);
  await expect(appearanceButton.locator('svg > *')).not.toHaveCount(0);
  const samples = await page.evaluate(
    () =>
      (window as Window & { __BDL_ICON_BUTTON_SAMPLES__?: Array<{ buttons: number; icons: number }> })
        .__BDL_ICON_BUTTON_SAMPLES__ ?? [],
  );
  expect(samples.length).toBeGreaterThan(0);
  expect(samples.some(({ buttons, icons }) => icons < buttons)).toBe(false);
});

for (const theme of ['light', 'dark'] as const) {
  for (const viewport of [
    { name: 'standard', width: 1280, height: 800 },
    { name: 'narrow', width: 900, height: 700 },
  ]) {
    test(`${theme} ${viewport.name} parse workspace`, async ({ page }) => {
      await page.setViewportSize(viewport);
      await installTauriMock(page, theme);
      await page.goto('/');
      await expect(page.getByRole('heading', { name: '解析链接' })).toBeVisible();
      await expect(page.getByRole('button', { name: '开始解析' })).toBeEnabled();
      await expect(page).toHaveScreenshot(`parse-${theme}-${viewport.name}.png`, {
        animations: 'disabled',
        caret: 'hide',
        maxDiffPixelRatio: 0.01,
      });
    });
  }
}

for (const theme of ['light', 'dark'] as const) {
  test(`${theme} standard paged parse result workspace`, async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await installTauriMock(page, theme);
    await page.goto('/');
    await page.getByLabel('Bilibili 链接或 BV / AV').fill('favorite:fixture');
    await page.getByRole('button', { name: '开始解析' }).click();

    await expect(page.getByRole('button', { name: '解析', exact: true })).toBeVisible();
    await expect(page).toHaveScreenshot(`parse-result-${theme}-standard.png`, {
      animations: 'disabled',
      caret: 'hide',
      maxDiffPixelRatio: 0.01,
    });
  });
}

test('light standard library folder detail workspace', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await installTauriMock(page, 'light', true, true);
  await page.goto('/');
  await page.getByRole('button', { name: '内容库 收藏与订阅' }).click();
  await page.getByRole('tab', { name: /订阅合集/ }).click();
  await page.getByRole('button', { name: '进入 双赢之路' }).click();

  await expect(page.locator('.library-folder-header').getByRole('heading', { name: '双赢之路' })).toBeVisible();
  await expect(page.locator('.library-folder-header').getByText('5 个视频', { exact: true })).toHaveCount(0);
  await expect(page.locator('.library-folder-header img')).toHaveCount(0);
  await expect(page.locator('.source-function-toolbar').getByText('已加载 5 / 5 项')).toBeVisible();
  await expect(page).toHaveScreenshot('library-folder-detail-light-standard.png', {
    animations: 'disabled',
    caret: 'hide',
    maxDiffPixelRatio: 0.01,
  });
});

test('native browser context menu is suppressed', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await expect(page.getByRole('heading', { name: '解析链接' })).toBeVisible();

  const contextMenuAllowed = await page.evaluate(() =>
    document.body.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, cancelable: true })),
  );

  expect(contextMenuAllowed).toBe(false);
});

test('parse source can be submitted with Ctrl+Enter', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.goto('/');

  const input = page.getByLabel('Bilibili 链接或 BV / AV');
  await input.fill('BV1xx411c7mD');
  await input.press('Control+Enter');

  await expect(page.getByRole('row', { name: /测试视频/ })).toBeVisible();
});

test('the QR placeholder starts a session without a duplicate footer action', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await expect(page.getByRole('heading', { name: '解析链接' })).toBeVisible();

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
    () =>
      ((window as unknown as { __BDL_TEST_INVOKES__: string[] }).__BDL_TEST_INVOKES__ ?? []).filter(
        (command) => command === 'environment_health',
      ).length,
  );
  expect(environmentChecks).toBe(1);
});

test('settings header is concise and existing outputs are skipped by default', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await page.getByRole('button', { name: '设置 偏好与维护' }).click();

  await expect(page.getByRole('heading', { name: '设置' })).toBeVisible();
  const settingsHeading = page.locator('.settings-heading');
  await expect(settingsHeading.getByText('偏好与维护', { exact: true })).toHaveCount(0);
  await expect(settingsHeading.getByText(/调整新任务的默认行为/)).toHaveCount(0);
  await page.getByRole('button', { name: /文件命名/ }).click();
  await expect(page.getByLabel('重名处理')).toContainText('已有文件则跳过');
});

test('about brand copy forms a compact stack beside the icon', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await page.getByRole('button', { name: '关于 版本与链接' }).click();

  const header = page.locator('.about-header');
  const brand = header.getByText('BILIBILI DOWNLOAD LAB', { exact: true });
  const heading = page.getByRole('heading', { name: 'BDL' });
  const tagline = page.getByText('一个专注解析、选择和稳定下载的 Bilibili 桌面工具。');
  await expect(brand).toBeVisible();
  await expect(heading).toBeVisible();
  await expect(tagline).toBeVisible();
  const layout = await heading.evaluate((element) => {
    const copy = element.parentElement;
    const header = copy?.parentElement;
    const brandElement = copy?.querySelector<HTMLElement>('.about-brand');
    const taglineElement = copy?.querySelector<HTMLElement>('.about-tagline');
    const iconElement = header?.querySelector<HTMLElement>('.about-icon');
    if (!copy || !header || !brandElement || !taglineElement || !iconElement) {
      throw new Error('Missing about brand layout elements');
    }
    const brandRect = brandElement.getBoundingClientRect();
    const headingRect = element.getBoundingClientRect();
    const taglineRect = taglineElement.getBoundingClientRect();
    const iconRect = iconElement.getBoundingClientRect();
    const copyRect = copy.getBoundingClientRect();
    return {
      copyIsStack: getComputedStyle(copy).display === 'grid',
      brandAboveTitle: brandRect.bottom <= headingRect.top,
      taglineBelowTitle: taglineRect.top >= headingRect.bottom,
      copyBesideIcon: copyRect.left > iconRect.right,
    };
  });
  expect(layout).toEqual({
    copyIsStack: true,
    brandAboveTitle: true,
    taglineBelowTitle: true,
    copyBesideIcon: true,
  });
  await expect(page).toHaveScreenshot('about-light-standard.png', {
    animations: 'disabled',
    caret: 'hide',
    maxDiffPixelRatio: 0.01,
  });
});

test('collapsed navigation centers the status indicator in a full navigation row', async ({ page }) => {
  await page.setViewportSize({ width: 900, height: 700 });
  await installTauriMock(page, 'light');
  await page.goto('/');

  const alignment = await page.locator('.nav-status').evaluate((status) => {
    const beacon = status.querySelector<HTMLElement>('.status-beacon');
    if (!beacon) throw new Error('Missing navigation status beacon');
    const statusRect = status.getBoundingClientRect();
    const beaconRect = beacon.getBoundingClientRect();
    return {
      height: statusRect.height,
      horizontalOffset: Math.abs(statusRect.left + statusRect.width / 2 - (beaconRect.left + beaconRect.width / 2)),
      verticalOffset: Math.abs(statusRect.top + statusRect.height / 2 - (beaconRect.top + beaconRect.height / 2)),
    };
  });
  expect(alignment.height).toBeGreaterThanOrEqual(54);
  expect(alignment.horizontalOffset).toBeLessThanOrEqual(1);
  expect(alignment.verticalOffset).toBeLessThanOrEqual(1);
});

test('opening download settings reuses startup environment health', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await expect(page.getByRole('button', { name: '开始解析' })).toBeEnabled();

  await page.getByLabel('链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();
  await page.getByRole('row', { name: /测试视频/ }).click();
  await page.getByRole('button', { name: '下载所选 (1)' }).click();

  await expect(page.getByRole('dialog', { name: '下载设置' })).toBeVisible();
  const environmentChecks = await page.evaluate(
    () =>
      ((window as unknown as { __BDL_TEST_INVOKES__: string[] }).__BDL_TEST_INVOKES__ ?? []).filter(
        (command) => command === 'environment_health',
      ).length,
  );
  expect(environmentChecks).toBe(1);
});

test('parse result selection controls live in the shared bottom action bar', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await page.getByLabel('Bilibili 链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();

  const table = page.getByRole('table', { name: '解析结果' });
  await expect(table.getByRole('columnheader', { name: '序号' })).toBeVisible();
  await expect(table.getByRole('columnheader', { name: '标题' })).toBeVisible();
  await expect(table.getByRole('columnheader', { name: 'UP 主' })).toBeVisible();
  await table.getByRole('checkbox', { name: '全选已加载' }).click();
  await expect(table.getByRole('checkbox', { name: '全选已加载' })).toBeChecked();
  const actionBar = page.locator('.selection-action-bar');
  await expect(actionBar.getByRole('button', { name: /全选/ })).toHaveCount(0);
  await expect(actionBar.getByRole('button', { name: /下载/ })).toHaveCount(0);
  await expect(page.locator('.source-function-toolbar').getByRole('button', { name: '下载所选 (1)' })).toBeVisible();
  await expect(page.locator('.source-result-header').getByRole('button', { name: /全选/ })).toHaveCount(0);
});

test('parse result uses a back action that clears the workspace and removes advanced filters', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  const input = page.getByLabel('Bilibili 链接或 BV / AV');
  await input.fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();

  await expect(page.getByRole('button', { name: '返回解析首页' })).toBeVisible();
  await expect(page.getByLabel('搜索内容')).toHaveCount(0);
  await expect(page.getByLabel('排序')).toHaveCount(0);
  await expect(page.getByLabel('序号范围')).toHaveCount(0);
  await expect(page.getByRole('button', { name: '刷新当前来源' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: '关闭解析结果' })).toHaveCount(0);
  await expect(page.locator('.source-function-toolbar').getByRole('button', { name: '下载所选 (0)' })).toBeVisible();

  await page.getByRole('button', { name: '返回解析首页' }).click();
  await expect(page.getByRole('heading', { name: '解析链接' })).toBeVisible();
  await expect(input).toHaveValue('');
  await expect(page.getByText('测试视频', { exact: true })).toHaveCount(0);
});

test('parse all waits between batches and can be stopped from the bottom action bar', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await page.getByLabel('Bilibili 链接或 BV / AV').fill('favorite:fixture');
  await page.getByRole('button', { name: '开始解析' }).click();

  const toolbar = page.locator('.source-function-toolbar');
  await expect(toolbar.getByRole('button', { name: '解析', exact: true })).toBeVisible();
  await toolbar.getByRole('button', { name: '更多解析方式' }).click();
  await page.getByRole('menuitem', { name: '解析全部' }).click();
  await expect(toolbar.getByText('等待 3 秒后继续…')).toBeVisible();
  await toolbar.getByRole('button', { name: '停止解析' }).click();
  await expect(toolbar.getByRole('button', { name: '解析', exact: true })).toBeVisible();

  const loadAllCalls = await page.evaluate(
    () =>
      ((window as unknown as { __BDL_TEST_INVOKES__: string[] }).__BDL_TEST_INVOKES__ ?? []).filter(
        (command) => command === 'parse_load_all',
      ).length,
  );
  expect(loadAllCalls).toBe(1);
});

test('parse and download loads every remaining batch before opening download settings', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await page.getByLabel('Bilibili 链接或 BV / AV').fill('favorite:fixture');
  await page.getByRole('button', { name: '开始解析' }).click();

  const toolbar = page.locator('.source-function-toolbar');
  await toolbar.getByRole('button', { name: '更多解析方式' }).click();
  await page.getByRole('menuitem', { name: '解析后下载' }).click();

  const dialog = page.getByRole('dialog', { name: '下载设置' });
  await expect(dialog).toBeVisible({ timeout: 6_000 });
  await expect(dialog.getByText('个分集将加入传输')).toBeVisible();
  await expect(dialog.getByText('9', { exact: true })).toBeVisible();
});

test('batch result selection controls use the shared bottom action bar', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await page.getByLabel('Bilibili 链接或 BV / AV').fill('BV1xx411c7mD\nBV1xx411c7mE');
  await page.getByRole('button', { name: '开始解析' }).click();

  const actionBar = page.locator('.selection-action-bar');
  await expect(actionBar.getByRole('button', { name: '取消全选' })).toBeVisible();
  await expect(actionBar.getByRole('button', { name: /下载/ })).toHaveCount(0);
  await expect(page.locator('.source-function-toolbar').getByRole('button', { name: '下载所选 (2)' })).toBeVisible();
  await expect(page.locator('.batch-result-header').getByRole('button', { name: /全选/ })).toHaveCount(0);
});

test('library uses its title link and makes the whole folder card the enter action', async ({ page }) => {
  await installTauriMock(page, 'light', true, true);
  await page.goto('/');
  await page.getByRole('button', { name: '内容库 收藏与订阅' }).click();

  const categoryLink = page.getByRole('button', { name: '在 Bilibili 打开我的收藏夹' });
  await expect(categoryLink).toHaveText('收藏夹');
  await expect(categoryLink.locator('svg')).toHaveCount(0);
  await expect(page.getByRole('button', { name: '在 Bilibili 打开 Web' })).toHaveCount(0);
  const enterFolder = page.getByRole('button', { name: '进入 Web' });
  await expect(enterFolder).toHaveClass(/library-card/);
  await expect(enterFolder.locator('.ui-icon-button')).toHaveCount(0);

  await enterFolder.click();
  await expect(page.getByRole('button', { name: '返回内容集合' })).toBeVisible();
});

test('library list reserves a dedicated gutter for its vertical scrollbar', async ({ page }) => {
  await page.setViewportSize({ width: 900, height: 380 });
  await installTauriMock(page, 'light', true, true);
  await page.goto('/');
  await page.keyboard.press('Control+2');

  const metrics = await page.locator('.library-content').evaluate((container) => {
    const card = container.querySelector<HTMLElement>('.library-card');
    if (!card) throw new Error('Missing library card');
    const containerRect = container.getBoundingClientRect();
    const cardRect = card.getBoundingClientRect();
    return {
      scrollbarGutter: getComputedStyle(container).scrollbarGutter,
      cardToContainerEdge: containerRect.right - cardRect.right,
    };
  });

  expect(metrics.scrollbarGutter).toContain('stable');
  expect(metrics.cardToContainerEdge).toBeGreaterThanOrEqual(6);
});

test('task-creation errors stay inside download settings instead of the parse page', async ({ page }) => {
  await installTauriMock(page, 'light', true, false, true);
  await page.goto('/');
  await expect(page.getByRole('button', { name: '开始解析' })).toBeEnabled();

  await page.getByLabel('链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();
  await page.getByRole('row', { name: /测试视频/ }).click();
  await page.getByRole('button', { name: '下载所选 (1)' }).click();

  const dialog = page.getByRole('dialog', { name: '下载设置' });
  await dialog.getByRole('button', { name: '加入传输' }).click();
  await expect(dialog.getByText(/Bilibili 暂时拒绝了请求/)).toBeVisible();

  await dialog.getByRole('button', { name: '取消' }).click();
  await expect(page.getByText(/Bilibili 暂时拒绝了请求/)).toHaveCount(0);
});

test('collected favorite folders open through the collection resolver with matching totals', async ({ page }) => {
  await installTauriMock(page, 'light', true, true);
  await page.goto('/');
  await page.getByRole('button', { name: '内容库 收藏与订阅' }).click();
  await page.getByRole('tab', { name: /订阅合集/ }).click();

  const enterFolder = page.getByRole('button', { name: '进入 双赢之路' });
  await expect(enterFolder).toBeVisible();
  await enterFolder.click();

  await expect(page.locator('.source-function-toolbar').getByText('已加载 5 / 5 项')).toBeVisible();
  const actionBar = page.locator('.selection-action-bar');
  await expect(actionBar.getByRole('button', { name: '全选本页' })).toBeVisible();
  await expect(actionBar.getByRole('button', { name: /下载|解析/ })).toHaveCount(0);
  await expect(page.locator('.library-folder-header').getByRole('button', { name: '下载全部' })).toBeVisible();
});

test('subscription collection cards fall back to the collection owner when archive authors are missing', async ({ page }) => {
  await installTauriMock(page, 'light', true, true);
  await page.goto('/');
  await page.getByRole('button', { name: '内容库 收藏与订阅' }).click();
  await page.getByRole('tab', { name: /订阅合集/ }).click();
  await page.getByRole('button', { name: '进入 双赢之路' }).click();

  const firstCard = page.locator('.library-video-card').first();
  await expect(firstCard.getByText('合集作者', { exact: true })).toBeVisible();
  await expect(firstCard.getByText('未知 UP 主', { exact: true })).toHaveCount(0);
});

test('leaving a library folder does not leak its parse session into the parse workspace', async ({ page }) => {
  await installTauriMock(page, 'light', true, true);
  await page.goto('/');
  await page.getByRole('button', { name: '内容库 收藏与订阅' }).click();
  await page.getByRole('tab', { name: /订阅合集/ }).click();
  await page.getByRole('button', { name: '进入 双赢之路' }).click();
  await expect(page.locator('.source-function-toolbar').getByText('已加载 5 / 5 项')).toBeVisible();

  await page.getByRole('button', { name: '解析 添加与选择' }).click();

  await expect(page.getByRole('heading', { name: '解析链接' })).toBeVisible();
  await expect(page.getByRole('heading', { name: /双赢之路/ })).toHaveCount(0);
  await expect(page.getByRole('table', { name: '解析结果' })).toHaveCount(0);
});

test('library detail keeps pagination, page count, parsing, and selection on one compact footer row', async ({ page }) => {
  await page.setViewportSize({ width: 1080, height: 720 });
  await installTauriMock(page, 'light', true, true);
  await page.goto('/');
  await page.keyboard.press('Control+2');
  await page.getByRole('tab', { name: /订阅合集/ }).click();
  await page.getByRole('button', { name: '进入 双赢之路' }).click();

  const actionBar = page.locator('.selection-action-bar');
  await expect(actionBar.getByText('第 1 / 1 页')).toBeVisible();
  await expect(page.getByText('第 1 / 1 页')).toHaveCount(1);
  await expect(page.locator('.source-function-toolbar').getByText('已加载 5 / 5 项')).toBeVisible();
  await expect(actionBar.locator('.selection-context-row')).toHaveCount(0);
  await expect(actionBar.getByRole('button', { name: /下载|解析/ })).toHaveCount(0);
  const footerHeight = await actionBar.evaluate((footer) => footer.getBoundingClientRect().height);
  expect(footerHeight).toBeLessThanOrEqual(44);
  await expect(page).toHaveScreenshot('library-folder-detail-light-compact.png', {
    animations: 'disabled',
    caret: 'hide',
    maxDiffPixelRatio: 0.01,
  });
});

test('folder detail opens immediately and shows a skeleton while collection data loads', async ({ page }) => {
  await installTauriMock(page, 'light', true, true, false, 1_000);
  await page.goto('/');
  await page.getByRole('button', { name: '内容库 收藏与订阅' }).click();
  await page.getByRole('tab', { name: /订阅合集/ }).click();

  await page.getByRole('button', { name: '进入 双赢之路' }).click();

  await expect(page.getByRole('button', { name: '返回内容集合' })).toBeVisible({ timeout: 250 });
  await expect(page.getByLabel('正在加载合集内容')).toBeVisible({ timeout: 250 });
  await expect(page.locator('.source-function-toolbar').getByText('已加载 5 / 5 项')).toBeVisible();
});

test('folder and video titles are the only Bilibili links inside detail cards', async ({ page }) => {
  await installTauriMock(page, 'light', true, true);
  await page.goto('/');
  await page.getByRole('button', { name: '内容库 收藏与订阅' }).click();
  await page.getByRole('tab', { name: /订阅合集/ }).click();
  await page.getByRole('button', { name: '进入 双赢之路' }).click();

  const folderTitleLink = page.getByRole('button', { name: '在 Bilibili 打开 双赢之路' });
  const videoTitleLink = page.getByRole('button', { name: '在 Bilibili 打开 合集视频 1' });
  await expect(folderTitleLink.locator('svg')).toHaveCount(0);
  await expect(videoTitleLink.locator('svg')).toHaveCount(0);
  await expect(page.getByRole('button', { name: /打开测试用户|打开 测试用户/ })).toHaveCount(0);
});

test('library video cards use checkbox selection and keep the bottom action bar aligned', async ({ page }) => {
  await installTauriMock(page, 'light', true, true);
  await page.goto('/');
  await page.getByRole('button', { name: '内容库 收藏与订阅' }).click();
  await page.getByRole('tab', { name: /订阅合集/ }).click();
  await page.getByRole('button', { name: '进入 双赢之路' }).click();

  const firstCard = page.locator('.library-video-card').first();
  await expect(firstCard).toHaveAttribute('data-selected', 'false');
  await expect(firstCard.getByRole('checkbox', { name: '选择 合集视频 1' })).not.toBeChecked();

  await firstCard.locator('.library-video-card-owner').click();
  await expect(firstCard).toHaveAttribute('data-selected', 'true');
  await expect(firstCard.getByRole('checkbox', { name: '选择 合集视频 1' })).toBeChecked();

  await firstCard.getByRole('checkbox', { name: '选择 合集视频 1' }).click();
  await expect(firstCard).toHaveAttribute('data-selected', 'false');
  await firstCard.getByRole('checkbox', { name: '选择 合集视频 1' }).click();
  await expect(firstCard).toHaveAttribute('data-selected', 'true');
  const selectedVisualState = await firstCard.evaluate((card) => {
    const selector = card.querySelector<HTMLElement>('.library-card-selector');
    if (!selector) throw new Error('Missing .library-card-selector');
    const cardStyle = getComputedStyle(card);
    const selectorStyle = getComputedStyle(selector);
    return {
      cardOutlineStyle: cardStyle.outlineStyle,
      cardBoxShadow: cardStyle.boxShadow,
      selectorBackground: selectorStyle.backgroundColor,
      selectorBorderWidth: selectorStyle.borderTopWidth,
    };
  });
  expect(selectedVisualState).toEqual({
    cardOutlineStyle: 'none',
    cardBoxShadow: 'none',
    selectorBackground: 'rgba(0, 0, 0, 0)',
    selectorBorderWidth: '0px',
  });
  await page.getByRole('button', { name: '全选本页' }).click();
  await expect(page.getByRole('button', { name: '取消本页选择' })).toBeVisible();

  const verticalCenters = await page.locator('.selection-action-bar').evaluate((footer) => {
    const center = (selector: string) => {
      const element = footer.querySelector<HTMLElement>(selector);
      if (!element) throw new Error(`Missing ${selector}`);
      const rect = element.getBoundingClientRect();
      return rect.top + rect.height / 2;
    };
    return [
      center('.library-pagination-status'),
      center('.selection-control-group'),
    ];
  });
  expect(Math.max(...verticalCenters) - Math.min(...verticalCenters)).toBeLessThanOrEqual(1);
});
