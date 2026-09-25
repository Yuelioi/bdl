import { expect, test, type Page } from '@playwright/test';
import type { NormalizedSourceTree } from '../../src/api/dto';

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
                usage_notice_acknowledged: true,
                auto_check_updates: false,
                theme_preference: selectedTheme,
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
            if (command === 'parse_load_more' && args?.request?.source_id === 'favorite:fixture') {
              await new Promise((resolve) => setTimeout(resolve, 100));
              pagedFixtureLoaded = Math.min(pagedFixtureLoaded + 20, 401);
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

test('mobile shell keeps navigation and downloads usable without desktop steps', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.addInitScript(() => {
    Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' });
  });
  await installTauriMock(page, 'light', true, true);
  await page.goto('/');
  await expect(page.locator('.app-mobile')).toBeVisible();
  await expect(page.getByRole('navigation', { name: '主导航' }).getByRole('button')).toHaveCount(4);
  await expect(page.locator('aside.side-nav')).toHaveCount(0);
  await expect(page.getByRole('navigation', { name: '操作阶段' })).toHaveCount(0);
  await expect(page.locator('.mobile-header')).toBeVisible();
  expect((await page.locator('.mobile-header').boundingBox())!.height).toBeLessThanOrEqual(52);
  await expect(page.locator('.mobile-header .account-button')).toHaveCount(0);
  await expect(page.locator('.mobile-header').getByRole('button')).toHaveCount(0);
  await page.getByLabel('Bilibili 链接或 BV / AV').fill('favorite:fixture');
  await page.getByRole('button', { name: '开始解析' }).click();
  const results = page.getByRole('list', { name: '解析结果' });
  await expect(results).toBeVisible();
  const parseRow = results.getByRole('listitem').first();
  expect((await parseRow.boundingBox())!.height).toBeLessThanOrEqual(64);
  expect((await parseRow.locator('.result-cover').boundingBox())!.height).toBeLessThanOrEqual(49);
  expect(await parseRow.locator('.result-cover').evaluate((cover) => getComputedStyle(cover).borderRadius)).toBe('4px');
  expect(await parseRow.locator('.result-copy strong').evaluate((title) => getComputedStyle(title).fontSize)).toBe('13px');
  await expect(parseRow.locator('.result-copy').locator(':scope > *')).toHaveCount(2);
  await results.getByRole('listitem').first().locator('.mobile-result-content').click();
  await expect(page.locator('.mobile-download-bar').getByRole('button', { name: '下载所选 (1)' })).toBeEnabled();
  await page.getByRole('navigation', { name: '主导航' }).getByRole('button', { name: '我的' }).click();
  await expect(page.locator('.profile-card')).toBeVisible();
  await expect(page.getByRole('heading', { name: '我的', exact: true })).toHaveCount(0);
  await expect(page.getByRole('button', { name: /外观：/ })).toBeVisible();
  await page.getByRole('navigation', { name: '主导航' }).getByRole('button', { name: '解析' }).click();
  await expect(page.locator('.mobile-download-bar').getByRole('button', { name: '下载所选 (1)' })).toBeEnabled();
  await expect(page.locator('.workspace-enter-active, .workspace-leave-active')).toHaveCount(0);
  for (const width of [390, 320]) {
    await page.setViewportSize({ width, height: 844 });
    const footer = await page.locator('.mobile-download-bar').boundingBox();
    const action = await page.locator('.mobile-download-bar .ui-button').boundingBox();
    const nav = await page.locator('.mobile-navigation').boundingBox();
    expect(footer!.y + footer!.height).toBeLessThanOrEqual(nav!.y);
    expect(action!.height).toBeLessThanOrEqual(32);
    expect(action!.width).toBeLessThanOrEqual(112);
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(width);
    await page.screenshot({ path: testInfo.outputPath(`mobile-result-${width}.png`) });
  }
  await page.getByRole('button', { name: '下载所选 (1)' }).click();
  const downloadDialog = page.getByRole('dialog');
  await expect(downloadDialog).toBeVisible();
  const cancelBox = await downloadDialog.getByRole('button', { name: '取消', exact: true }).boundingBox();
  const startBox = await downloadDialog.getByRole('button', { name: '开始下载' }).boundingBox();
  expect(cancelBox!.x).toBeLessThan(startBox!.x);
  await page.screenshot({ path: testInfo.outputPath('mobile-download-dialog.png') });
});

test('mobile library uses compact covers and shared page actions', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.addInitScript(() => Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }));
  await installTauriMock(page, 'light', true, true);
  await page.addInitScript(() => {
    const target = window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> } };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      if (command === 'settings_update') return (args as { settings: unknown }).settings;
      const result = await invoke(command, args);
      if (command === 'parse_create_source') {
        const tree = result as NormalizedSourceTree;
        tree.groups?.forEach(group => group.items.forEach((item, index) => {
          item.cover_url = index === 0 ? 'http://i0.hdslb.com/test-cover.svg' : null;
        }));
      }
      return result;
    };
  });
  const referers: Array<string | undefined> = [];
  await page.route('https://i0.hdslb.com/test-cover.svg', async route => {
    referers.push(route.request().headers().referer);
    await route.fulfill({ contentType: 'image/svg+xml', body: '<svg xmlns="http://www.w3.org/2000/svg" width="160" height="100"><rect width="160" height="100" fill="#c7d8cf"/></svg>' });
  });
  await page.goto('/');
  const nav = page.getByRole('navigation', { name: '主导航' });
  await nav.getByRole('button', { name: '内容库' }).click();
  await page.getByRole('tab', { name: /订阅合集/ }).click();
  await page.getByRole('button', { name: '进入 双赢之路' }).click();
  await expect(page.locator('.folder-heading')).toContainText('双赢之路');
  const cover = page.locator('.mobile-result img').first();
  await expect(cover).toHaveAttribute('src', 'https://i0.hdslb.com/test-cover.svg');
  await expect.poll(() => cover.evaluate((image: HTMLImageElement) => image.naturalWidth)).toBe(160);
  expect(referers).toEqual([undefined]);
  const folderRow = page.locator('.mobile-result').first();
  expect((await folderRow.boundingBox())!.height).toBeLessThanOrEqual(64);
  expect((await folderRow.locator('.result-cover').boundingBox())!.height).toBeLessThanOrEqual(49);
  expect(await folderRow.locator('.result-cover').evaluate((artwork) => getComputedStyle(artwork).borderRadius)).toBe('4px');
  await page.locator('.mobile-result-content').first().click();
  await expect(page.locator('.folder-download').getByRole('button', { name: '下载所选 (1)' })).toBeEnabled();
  for (const width of [390, 320]) {
    await page.setViewportSize({ width, height: 844 });
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(width);
    const footer = await page.locator('.folder-download').boundingBox();
    const action = await page.locator('.folder-download .ui-button').boundingBox();
    const navigation = await nav.boundingBox();
    expect(footer!.y + footer!.height).toBeLessThanOrEqual(navigation!.y);
    expect(action!.height).toBeLessThanOrEqual(32);
    expect(action!.width).toBeLessThanOrEqual(112);
    await page.screenshot({ path: testInfo.outputPath(`mobile-library-${width}.png`) });
  }
  await page.locator('.folder-download').getByRole('button').click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.keyboard.press('Escape');
  await nav.getByRole('button', { name: '我的' }).click();
  await expect(page.locator('.profile-card')).toBeVisible();
  await expect(page.locator('.workspace-enter-active, .workspace-leave-active')).toHaveCount(0);
  await page.screenshot({ path: testInfo.outputPath('reference-personal.png') });
  await page.getByRole('button', { name: /下载设置/ }).click();
  await page.getByRole('button', { name: '下载 目录、并发与恢复' }).click();
  await page.getByLabel('全局下载限速（MiB/s）', { exact: true }).fill('10');
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByRole('button', { name: '保存', exact: true })).toBeDisabled();
  await page.screenshot({ path: testInfo.outputPath('mobile-settings.png') });
  await nav.getByRole('button', { name: '传输' }).click();
  await expect(page.locator('.transfer-main')).toBeVisible();
  await page.screenshot({ path: testInfo.outputPath('mobile-transfer.png') });
  await nav.getByRole('button', { name: '我的' }).click();
  await page.getByRole('button', { name: /关于 BDL/ }).click();
  await expect(page.getByText('GitHub 仓库')).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(320);
  await page.screenshot({ path: testInfo.outputPath('mobile-about.png') });
});

test('mobile library collection index stays flat and two-line in dark theme', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 320, height: 844 });
  await page.addInitScript(() =>
    Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }),
  );
  await installTauriMock(page, 'dark', true, true);
  await page.goto('/');
  await page.getByRole('navigation', { name: '主导航' }).getByRole('button', { name: '内容库' }).click();
  await page.getByRole('tab', { name: /订阅合集/ }).click();

  const cards = page.locator('.library-card');
  await expect(cards.first()).toBeVisible();
  await expect(cards.locator('.library-description')).toHaveCount(0);
  for (const card of await cards.all()) {
    expect((await card.boundingBox())!.height).toBeLessThanOrEqual(64);
    await expect(card.locator('.library-card-copy').locator(':scope > *')).toHaveCount(2);
    expect(await card.evaluate((element) => getComputedStyle(element).backgroundColor)).toBe('rgba(0, 0, 0, 0)');
  }
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(320);
  await page.screenshot({ path: testInfo.outputPath('mobile-library-index-dark.png') });
});

test('mobile redesign keeps content first with populated lists and bottom sheets', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.addInitScript(() => Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }));
  await installTauriMock(page, 'light', true, true);
  await page.addInitScript(() => {
    const target = window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> } };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      if (command === 'settings_update') return (args as { settings: unknown }).settings;
      if (command === 'queue_logs') return [];
      if (command === 'queue_list') return Array.from({ length: 123 }, (_, index) => ({
        id: `mobile-task-${index}`, title: ['测试屏幕 · 壁纸级 4K 8K HDR 画面', 'bilibili 8K 带你看亚洲 33 个国家', '异环高效锄地路线 1'][index % 3],
        source_id: 'video:fixture', status: index === 0 ? 'downloading' : index === 1 ? 'paused' : index === 2 ? 'failed' : 'completed', resources: index === 0 ? [{ id: 'resource:video', kind: 'video', intent: 'video', current_urls: [], headers: [], status: 'downloading', target_path: 'downloads/video.m4s', temp_path: 'downloads/video.m4s.part' }] : [], output_path: 'downloads/测试视频.mp4', refresh_intent: { input: { kind: 'video_bvid', bvid: 'BV1fixture' }, cid: index + 1, cover_url: 'http://i0.hdslb.com/transfer-cover.svg', duration_seconds: 60 }, media_selection: null, scheduled_at: null, speed_limit_bytes_per_second: null,
      }));
      const result = await invoke(command, args);
      if (command === 'account_library_list') {
        const list = result as { items: Array<{ title: string; source_url: string; media_count: number }>; total: number };
        list.total = 33;
        list.items[0] = { ...list.items[0], title: '默认收藏夹', source_url: 'favorite:fixture', media_count: 356 };
      }
      if (command === 'parse_create_source') {
        const tree = result as NormalizedSourceTree;
        tree.source.title = '默认收藏夹'; tree.source.total_count = 356; tree.source.loaded_count = 40;
        const base = tree.groups[0].items[0];
        tree.groups[0].items = Array.from({ length: 40 }, (_, index) => ({ ...base, id: `item-${index}`, title: ['测试屏幕 · 壁纸级 4K 8K HDR 画面', 'bilibili 8K 带你看亚洲 33 个国家', '异环高效锄地路线 1'][index % 3], parts: [{ ...base.parts[0], id: `part-${index}` }] }));
      }
      return result;
    };
  });
  await page.route('https://i0.hdslb.com/transfer-cover.svg', (route) =>
    route.fulfill({
      contentType: 'image/svg+xml',
      body: '<svg xmlns="http://www.w3.org/2000/svg" width="160" height="90"><rect width="160" height="90" fill="#d8c9d1"/></svg>',
    }),
  );
  await page.goto('/');
  const expectSheetInViewport = async (name: string) => {
    const viewport = page.viewportSize();
    expect(viewport).not.toBeNull();
    await expect
      .poll(async () => {
        const sheet = await page.getByRole('dialog', { name }).boundingBox();
        return {
          left: Math.round(sheet!.x),
          right: Math.round(sheet!.x + sheet!.width),
          bottom: Math.round(sheet!.y + sheet!.height),
        };
      })
      .toEqual({ left: 0, right: viewport!.width, bottom: viewport!.height });
  };
  await expect(page.getByRole('button', { name: '开始解析' })).toBeVisible();
  await page.screenshot({ path: testInfo.outputPath('redesign-parse.png') });
  const nav = page.getByRole('navigation', { name: '主导航' });
  await nav.getByRole('button', { name: '内容库' }).click();
  const firstCollection = page.locator('.library-card').first();
  const firstCollectionCover = firstCollection.locator('.library-cover');
  await expect(firstCollection.locator('.library-description')).toHaveCount(0);
  await expect(firstCollection.locator('.library-card-copy').locator(':scope > *')).toHaveCount(2);
  expect((await firstCollectionCover.boundingBox())!.height).toBeLessThanOrEqual(49);
  expect(await firstCollectionCover.evaluate((cover) => getComputedStyle(cover).borderRadius)).toBe('4px');
  expect(await firstCollection.locator('h3').evaluate((title) => getComputedStyle(title).fontSize)).toBe('13px');
  expect((await firstCollection.boundingBox())!.height).toBeLessThanOrEqual(64);
  await page.getByRole('button', { name: '进入 默认收藏夹' }).click();
  await expect(page.locator('.folder-heading')).toContainText('356 个视频');
  await expect(page.getByRole('tablist')).toHaveCount(0);
  await expect(page.locator('.source-parse-controls')).toHaveCount(0);
  await expect(page.getByRole('combobox')).toHaveCount(0);
  const pagination = page.getByRole('navigation', { name: '集合内容分页' });
  await expect(pagination.getByRole('button', { name: '上一页' })).toBeDisabled();
  await expect(pagination.getByRole('button', { name: '下一页' })).toBeEnabled();
  await expect(pagination.getByRole('textbox')).toHaveValue('1');
  for (const width of [390, 320]) {
    await page.setViewportSize({ width, height: 844 });
    const first = await page.locator('.mobile-result').first().boundingBox();
    expect(first!.y).toBeLessThan(220);
    const paginationBox = await pagination.boundingBox();
    const folderFooterBox = await page.locator('.folder-download').boundingBox();
    expect(paginationBox!.width).toBeLessThanOrEqual(width - 24);
    expect(paginationBox!.height).toBeLessThanOrEqual(32);
    expect(
      await pagination.evaluate((element) => {
        const page = element.closest('.mobile-page');
        return [getComputedStyle(element).backgroundColor, page ? getComputedStyle(page).backgroundColor : ''];
      }),
    ).toEqual(['rgb(251, 250, 251)', 'rgb(251, 250, 251)']);
    expect(paginationBox!.y + paginationBox!.height).toBeLessThanOrEqual(folderFooterBox!.y);
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(width);
    await page.screenshot({ path: testInfo.outputPath(`redesign-library-${width}.png`) });
  }
  await page.getByRole('button', { name: '内容操作' }).click();
  await expect(page.getByRole('dialog', { name: '内容操作' })).toBeVisible();
  await expect(page.getByLabel('每次加载')).toBeVisible();
  await expectSheetInViewport('内容操作');
  await page.screenshot({ path: testInfo.outputPath('redesign-source-sheet.png') });
  await page.keyboard.press('Escape');
  await nav.getByRole('button', { name: /传输/ }).click();
  await expect(page.getByRole('list', { name: '传输任务' })).toBeVisible();
  await expect(page.getByRole('table', { name: '传输任务' })).toHaveCount(0);
  await expect(page.locator('.task-playback-action').first()).toBeVisible();
  await expect(page.locator('.mobile-task').first()).toContainText('下载视频中');
  expect((await page.locator('.mobile-task').first().boundingBox())!.height).toBeLessThanOrEqual(64);
  expect((await page.locator('.task-artwork').first().boundingBox())!.height).toBeLessThanOrEqual(49);
  expect(await page.locator('.task-artwork').first().evaluate((artwork) => getComputedStyle(artwork).borderRadius)).toBe('4px');
  expect(await page.locator('.task-title').first().evaluate((title) => getComputedStyle(title).fontSize)).toBe('13px');
  expect(await page.locator('.task-playback-action').first().locator('svg').evaluate((icon) => icon.childElementCount)).toBeGreaterThan(0);
  const playbackBox = await page.locator('.task-playback-action').first().boundingBox();
  const durationBox = await page.locator('.mobile-task').first().locator('.artwork-duration').boundingBox();
  expect(playbackBox!.x + playbackBox!.width).toBeLessThan(durationBox!.x);
  await page.screenshot({ path: testInfo.outputPath('redesign-transfer-active.png') });
  await page.getByRole('tab', { name: /已完成/ }).click();
  await expect(page.getByRole('button', { name: '搜索已完成任务' })).toBeVisible();
  await expect(page.getByPlaceholder('搜索标题或保存位置')).toHaveCount(0);
  await page.getByRole('button', { name: '搜索已完成任务' }).click();
  await expect(page.getByPlaceholder('搜索标题或保存位置')).toBeVisible();
  await page.getByRole('button', { name: '关闭搜索' }).click();
  await expect(page.locator('.mobile-task img').first()).toHaveAttribute(
    'src',
    'https://i0.hdslb.com/transfer-cover.svg',
  );
  await expect(page.locator('.mobile-task').first().locator('.task-status-badge')).toHaveText('已完成');
  await expect(page.locator('.mobile-task').first().getByRole('button', { name: '播放' })).toBeVisible();
  await expect(page.locator('.mobile-task').first().getByText('打开文件', { exact: true })).toHaveCount(0);
  await expect(page.locator('.mobile-task').first().locator('.task-heading, .task-meta')).toHaveCount(2);
  await expect(page.locator('.mobile-header')).toBeVisible();
  expect((await page.locator('.mobile-header').boundingBox())!.height).toBeLessThanOrEqual(52);
  await expect(page.locator('.mobile-header')).toContainText('BDL');
  expect((await page.locator('.mobile-navigation').boundingBox())!.height).toBeLessThanOrEqual(60);
  expect((await page.locator('.mobile-task').first().boundingBox())!.height).toBeLessThanOrEqual(78);
  const taskListBox = await page.locator('.mobile-task-list').boundingBox();
  const taskListWidth = await page.locator('.mobile-task-list').evaluate((list) => ({
    client: list.clientWidth,
    scroll: list.scrollWidth,
  }));
  expect(taskListWidth.scroll).toBeLessThanOrEqual(taskListWidth.client);
  const fullyVisibleTaskCount = await page.locator('.mobile-task').evaluateAll((tasks, bounds) =>
    tasks.filter((task) => {
      const box = task.getBoundingClientRect();
      return box.top >= bounds.top && box.bottom <= bounds.bottom;
    }).length,
    { top: taskListBox!.y, bottom: taskListBox!.y + taskListBox!.height },
  );
  expect(fullyVisibleTaskCount).toBeGreaterThanOrEqual(8);
  for (const tab of await page.getByRole('tab').all()) {
    const box = await tab.boundingBox();
    expect(box!.height).toBeLessThanOrEqual(42);
    expect(box!.x + box!.width).toBeLessThanOrEqual(320);
  }
  const firstTask = page.locator('.mobile-task').first();
  const firstTaskBox = await firstTask.boundingBox();
  await page.mouse.move(firstTaskBox!.x + firstTaskBox!.width / 2, firstTaskBox!.y + firstTaskBox!.height / 2);
  await page.mouse.down();
  await page.waitForTimeout(460);
  await page.mouse.up();
  await expect(firstTask.getByRole('checkbox')).toBeChecked();
  await expect(page.getByRole('button', { name: '完成管理' })).toBeVisible();
  await page.screenshot({ path: testInfo.outputPath('redesign-transfer-selection.png') });
  await page.getByRole('button', { name: '完成管理' }).click();
  await expect(firstTask.getByRole('checkbox')).toHaveCount(0);
  await page.screenshot({ path: testInfo.outputPath('redesign-transfer.png') });
  await page.getByRole('button', { name: /：更多操作/ }).first().click();
  await expect(page.getByRole('dialog', { name: '任务操作' })).toBeVisible();
  await expectSheetInViewport('任务操作');
  await page.screenshot({ path: testInfo.outputPath('redesign-task-sheet.png') });
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: '传输选项' }).click();
  await expect(page.getByRole('dialog', { name: '传输选项' })).toBeVisible();
  await expectSheetInViewport('传输选项');
  await page.keyboard.press('Escape');
  await nav.getByRole('button', { name: '我的' }).click();
  await expect(page.locator('.profile-card')).toBeVisible();
  await expect(page.locator('.workspace-enter-active, .workspace-leave-active')).toHaveCount(0);
  await page.screenshot({ path: testInfo.outputPath('reference-personal.png') });
  await page.getByRole('button', { name: /下载设置/ }).click();
  await expect(page.locator('.settings-category')).toHaveCount(6);
  await expect(page.locator('.mobile-header')).toBeVisible();
  await page.screenshot({ path: testInfo.outputPath('redesign-settings.png') });
  await page.getByRole('button', { name: '下载 目录、并发与恢复' }).click();
  await expect(page.getByLabel('全局下载限速（MiB/s）', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: '返回设置' }).click();
  await expect(page.locator('.settings-category')).toHaveCount(6);
  await page.getByRole('button', { name: '返回我的' }).click();
  await page.getByRole('button', { name: /外观：/ }).first().click();
  await page.getByRole('menuitemcheckbox', { name: '深色' }).click();
  await page.keyboard.press('Escape');
  await expect(page.locator('html')).toHaveClass(/dark/);
  await expect(page.getByRole('alert')).toHaveCount(0);
  await page.screenshot({ path: testInfo.outputPath('reference-personal-dark.png') });
  await page.getByRole('button', { name: /下载设置/ }).click();
  await page.screenshot({ path: testInfo.outputPath('redesign-settings-dark.png') });
  await nav.getByRole('button', { name: /传输/ }).click();
  await expect(page.locator('.mobile-task-list')).toBeVisible();
  await expect(page.locator('.workspace-enter-active, .workspace-leave-active')).toHaveCount(0);
  await expect(page.getByRole('alert')).toHaveCount(0);
  await page.screenshot({ path: testInfo.outputPath('redesign-transfer-dark.png') });
});

test('mobile batch selection and removal keep the download action reachable', async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 844 });
  await page.addInitScript(() => Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }));
  await installTauriMock(page, 'light', true, true);
  await page.goto('/');
  await page.getByLabel('Bilibili 链接或 BV / AV').fill('BV1xx411c7mD\nBV1xx411c7mE');
  await page.getByRole('button', { name: '开始解析' }).click();
  const list = page.getByRole('list', { name: '批量解析结果' });
  await expect(list.getByRole('listitem')).toHaveCount(2);
  await list.getByRole('button', { name: '移除这个链接' }).last().click();
  await expect(list.getByRole('listitem')).toHaveCount(1);
  await list.getByRole('checkbox').first().setChecked(true);
  await expect(page.getByRole('button', { name: '下载所选 (1)' })).toBeEnabled();
  const footer = await page.locator('.mobile-download-footer').boundingBox();
  const nav = await page.locator('.mobile-navigation').boundingBox();
  expect(footer!.y + footer!.height).toBeLessThanOrEqual(nav!.y);
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(320);
});

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

test('single video mode uses one input and disables collection expansion', async ({ page }, testInfo) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await expect(page.getByRole('button', { name: '批量解析', exact: true })).toHaveAttribute('aria-pressed', 'true');
  await page.locator('textarea').fill('BV1xx411c7mD\nBV1xx411c7mE');
  await page.getByRole('button', { name: '单个视频', exact: true }).click();
  await expect(page.locator('textarea')).toHaveCount(0);
  const input = page.getByLabel('视频链接或 BV / AV', { exact: true });
  await input.fill('BV1xx411c7mD');
  await input.evaluate((element) => {
    const clipboardData = new DataTransfer();
    clipboardData.setData('text/plain', 'BV1xx411c7mD\nBV1xx411c7mE');
    element.dispatchEvent(new ClipboardEvent('paste', { clipboardData, bubbles: true, cancelable: true }));
  });
  await expect(page.getByText(/一次只解析一个链接/)).toBeVisible();
  await expect(input).toHaveValue('BV1xx411c7mD');
  await page.getByRole('button', { name: '批量解析', exact: true }).click();
  await expect(page.locator('textarea')).toHaveValue('BV1xx411c7mD\nBV1xx411c7mE');
  await page.getByRole('button', { name: '单个视频', exact: true }).click();
  await expect(input).toHaveValue('BV1xx411c7mD');
  await page.screenshot({ path: testInfo.outputPath('single-video.png') });
  await page.setViewportSize({ width: 540, height: 720 });
  await expect(input).toBeVisible();
  expect(await input.evaluate((element) => element.getBoundingClientRect().right)).toBeLessThanOrEqual(540);
  await page.screenshot({ path: testInfo.outputPath('single-video-narrow.png') });
  await page.evaluate(() => {
    const runtime = (window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> } }).__TAURI_INTERNALS__;
    const invoke = runtime.invoke;
    runtime.invoke = async (command, args) => {
      if (command === 'parse_create_source') Object.assign(window, { __SINGLE_REQUEST__: args });
      return invoke(command, args);
    };
  });
  await input.press('Enter');
  await expect(page.getByRole('row', { name: /测试视频/ })).toBeVisible();
  expect(await page.evaluate(() => (window as unknown as { __SINGLE_REQUEST__: unknown }).__SINGLE_REQUEST__)).toEqual({
    request: { input: 'BV1xx411c7mD', fetch_streams: false, expand_video_collection: false },
  });
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
  await expect(dialog.locator('.qr-box-action')).toHaveCount(0);

  await dialog.getByRole('button', { name: '刷新二维码' }).click();
  await expect(dialog.locator('.qr-box img')).toBeVisible();
  await expect(dialog.locator('.qr-box-action')).toHaveCount(0);
  await expect(dialog.getByRole('button', { name: '刷新二维码' })).toHaveCount(1);
});

test('startup environment warnings do not block parsing', async ({ page }) => {
  await installTauriMock(page, 'light', false);
  await page.goto('/');

  const dialog = page.getByRole('dialog', { name: '下载环境未就绪' });
  await expect(dialog).not.toBeVisible();
  await expect(page.getByRole('button', { name: '开始解析' })).toBeEnabled();
  await page.getByLabel('链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();
  await expect(page.getByRole('row', { name: /测试视频/ })).toBeVisible();

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
  await expect(actionBar.getByRole('button', { name: '取消全选', exact: true })).toBeVisible();
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

test('parse cooldown shows remaining seconds and stops immediately', async ({ page }, testInfo) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await page.getByLabel('链接或 BV / AV').fill('favorite:fixture');
  await page.getByRole('button', { name: '开始解析' }).click();
  await page.evaluate(() => {
    const runtime = (window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> } }).__TAURI_INTERNALS__;
    const invoke = runtime.invoke;
    let cancel: (() => void) | undefined;
    runtime.invoke = async (command, args) => {
      if (command === 'parse_load_more') return new Promise((_resolve, reject) => { cancel = () => reject(new Error('解析已停止')); });
      if (command === 'parse_progress') return { active: true, waiting_seconds: 3, queued: false };
      if (command === 'parse_cancel') { cancel?.(); return; }
      return invoke(command, args);
    };
  });
  const toolbar = page.locator('.source-function-toolbar');
  await toolbar.getByRole('button', { name: '解析', exact: true }).click();
  await expect(toolbar).toContainText('休息 3 秒后继续');
  await page.screenshot({ path: testInfo.outputPath('parse-cooldown.png') });
  await toolbar.getByRole('button', { name: '停止解析', exact: true }).click();
  await expect(toolbar.getByRole('button', { name: '解析', exact: true })).toBeVisible({ timeout: 1000 });
  await expect(toolbar).toContainText('9 / 401');
  await expect(page.getByText('解析已停止', { exact: true })).toHaveCount(0);
});

test('parse all can stop at a page boundary', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await page.getByLabel('链接或 BV / AV').fill('favorite:fixture');
  await page.getByRole('button', { name: '开始解析' }).click();
  const toolbar = page.locator('.source-function-toolbar');
  await expect(toolbar.getByRole('combobox', { name: '每批解析数量' })).toContainText('每批 50');
  await toolbar.getByRole('button', { name: '更多解析方式' }).click();
  await page.getByRole('menuitem', { name: '解析全部', exact: true }).click();
  await toolbar.getByRole('button', { name: '停止解析', exact: true }).click();
  await expect(toolbar.getByRole('button', { name: '解析', exact: true })).toBeVisible();
});

test('background parsing downloads progressively and remains visible on another page', async ({ page }, testInfo) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await page.getByLabel('链接或 BV / AV').fill('favorite:fixture');
  await page.getByRole('button', { name: '开始解析' }).click();
  await page.getByRole('button', { name: '更多解析方式' }).click();
  await page.getByRole('menuitem', { name: '后台解析全部并下载' }).click();
  await page.getByRole('button', { name: '设置 偏好与维护' }).click();
  const status = page.getByRole('region', { name: '后台解析下载' });
  await expect(status).toBeVisible();
  await expect(status).toContainText('后台解析下载已完成', { timeout: 15000 });
  await expect(page.getByRole('dialog', { name: '下载设置' })).not.toBeVisible();
  const commands = await page.evaluate(() => (window as unknown as { __BDL_TEST_INVOKES__: string[] }).__BDL_TEST_INVOKES__);
  expect(commands.indexOf('selection_create_tasks')).toBeLessThan(commands.indexOf('parse_load_more'));
  await page.screenshot({ path: testInfo.outputPath('background-download.png') });
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
  await dialog.getByRole('button', { name: '开始下载' }).click();
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

test('library page jump loads only the requested page and reuses visited pages', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1080, height: 720 });
  await installTauriMock(page, 'light', true, true);
  await page.goto('/');
  await page.evaluate(() => {
    const runtime = (window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: { request?: { page_number?: number } }) => Promise<unknown> } }).__TAURI_INTERNALS__;
    const invoke = runtime.invoke;
    let tree: NormalizedSourceTree;
    const requests: number[] = [];
    Object.assign(window, { __PAGE_REQUESTS__: requests });
    runtime.invoke = async (command, args) => {
      if (command === 'parse_load_more') throw new Error('Page navigation must not load intervening pages');
      if (command === 'parse_load_page') {
        const number = args?.request?.page_number ?? 1;
        requests.push(number);
        const item = structuredClone(tree.groups[0].items[0]);
        item.id = `item:page:${number}`;
        item.title = `第 ${number} 页视频`;
        item.parts[0].id = `part:page:${number}`;
        tree.groups[0].items.push(item);
        tree.source.loaded_count += 1;
        return { tree: structuredClone(tree), item_ids: [item.id] };
      }
      const result = await invoke(command, args);
      if (command === 'parse_create_source') {
        tree = result as NormalizedSourceTree;
        tree.source.total_count = 200;
        tree.source.has_more = true;
      }
      return result;
    };
  });
  await page.keyboard.press('Control+2');
  await page.getByRole('tab', { name: /订阅合集/ }).click();
  await page.getByRole('button', { name: '进入 双赢之路' }).click();
  const pagination = page.getByRole('navigation', { name: '集合内容分页' });
  await page.getByRole('button', { name: '全选本页', exact: true }).click();
  await pagination.getByRole('textbox').fill('8');
  await pagination.getByRole('textbox').press('Enter');
  await expect(page.getByText('第 8 页视频', { exact: true })).toBeVisible();
  await expect(page.locator('.library-selection-count')).toContainText('已选 5');
  await pagination.getByRole('textbox').fill('1');
  await pagination.getByRole('button', { name: '跳转', exact: true }).click();
  await expect(page.getByText('合集视频 1', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => (window as unknown as { __PAGE_REQUESTS__: number[] }).__PAGE_REQUESTS__)).toEqual([8]);
  await page.screenshot({ path: testInfo.outputPath('library-page-jump.png') });
  await page.setViewportSize({ width: 540, height: 720 });
  await expect(pagination.getByRole('textbox')).toBeVisible();
  expect(await pagination.evaluate((element) => element.getBoundingClientRect().right)).toBeLessThanOrEqual(540);
  await page.screenshot({ path: testInfo.outputPath('library-page-jump-narrow.png') });
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


test('media preferences reorder combinations while download keeps optimal quality', async ({ page }, testInfo) => {
  await installTauriMock(page, 'light');
  await page.addInitScript(() => {
    const target = window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> }; __BDL_MEDIA_REQUEST__?: unknown };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      if (command === 'settings_update') return args?.settings;
      if (command === 'selection_create_tasks') target.__BDL_MEDIA_REQUEST__ = args?.request;
      return invoke(command, args);
    };
  });
  await page.goto('/');
  await page.getByRole('button', { name: '设置 偏好与维护' }).click();
  await page.getByRole('button', { name: /媒体 清晰度/ }).click();
  await page.getByRole('button', { name: '添加画质' }).click();
  await page.getByRole('combobox', { name: '第 1 优先画质', exact: true }).click();
  await page.getByRole('option', { name: 'HDR / 125', exact: true }).click();
  await page.getByRole('combobox', { name: '第 1 优先编码', exact: true }).click();
  await page.getByRole('option', { name: 'HEVC / H.265', exact: true }).click();
  await page.getByRole('button', { name: '添加画质' }).click();
  await page.getByRole('combobox', { name: '第 2 优先画质', exact: true }).click();
  await page.getByRole('option', { name: '任意 SDR', exact: true }).click();
  await page.getByRole('button', { name: '上移第 2 条视频偏好', exact: true }).click();
  await expect(page.getByRole('combobox', { name: '第 1 优先画质', exact: true })).toContainText('任意 SDR');
  await page.getByRole('button', { name: '下移第 1 条视频偏好', exact: true }).click();
  await page.getByRole('button', { name: '添加音质' }).click();
  await page.getByRole('combobox', { name: '第 1 优先音质', exact: true }).click();
  await page.getByRole('option', { name: 'Hi-Res 无损 / 30251', exact: true }).click();
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByRole('button', { name: '保存', exact: true })).toBeDisabled();
  await page.screenshot({ path: testInfo.outputPath('media-preferences-light.png') });
  await page.setViewportSize({ width: 800, height: 900 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
  await page.screenshot({ path: testInfo.outputPath('media-preferences-narrow.png') });
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.getByRole('button', { name: '解析 添加与选择' }).click();
  await page.getByLabel('链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();
  await page.getByRole('row', { name: /测试视频/ }).click();
  await page.getByRole('button', { name: '下载所选 (1)' }).click();
  const dialog = page.getByRole('dialog', { name: '下载设置' });
  await dialog.getByRole('tab', { name: '画质与音频' }).click();
  await expect(dialog.getByRole('combobox', { name: /视频清晰度/ })).toContainText('最优画质');
  await expect(dialog.getByText('按本次优先顺序选择')).toBeVisible();
  await dialog.getByRole('button', { name: '开始下载' }).click();
  const request = await page.evaluate(() => (window as unknown as { __BDL_MEDIA_REQUEST__: { media_preferences: unknown } }).__BDL_MEDIA_REQUEST__);
  expect(request.media_preferences).toEqual({ video: [{ quality: '125', codec: 'hevc' }, { quality: 'sdr', codec: 'auto' }], audio: ['30251'], fallback: 'best' });
});


test('missing directory allows parsing and explicit SDR task creation', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.addInitScript(() => {
    const target = window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> }; __BDL_SDR_REQUEST__?: unknown };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      const result = await invoke(command, args);
      if (command === 'environment_health') {
        return { ...(result as object), ready: false, download_directory: { status: 'missing', path: 'C:\\Downloads\\new-folder', message: '开始下载时自动创建' } };
      }
      if (command === 'settings_get') {
        return { ...(result as object), media_preferences: { video: [{ quality: '125', codec: 'hevc' }], audio: ['30280'], fallback: 'best' } };
      }
      if (command === 'selection_create_tasks') target.__BDL_SDR_REQUEST__ = args?.request;
      return result;
    };
  });
  await page.goto('/');
  await page.getByLabel('链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();
  await page.getByRole('row', { name: /测试视频/ }).click();
  await page.getByRole('button', { name: '下载所选 (1)' }).click();
  const dialog = page.getByRole('dialog', { name: '下载设置' });
  await dialog.getByRole('tab', { name: '画质与音频' }).click();
  await dialog.getByRole('combobox', { name: /视频清晰度/ }).click();
  await page.getByRole('option', { name: 'SDR / 普通动态范围', exact: true }).click();
  await dialog.getByRole('button', { name: '开始下载' }).click();
  const result = await page.evaluate(() => {
    const target = window as unknown as { __BDL_SDR_REQUEST__: { quality: string; media_preferences: { video: unknown[]; audio: string[] } }; __BDL_TEST_INVOKES__: string[] };
    return { request: target.__BDL_SDR_REQUEST__, creates: target.__BDL_TEST_INVOKES__.filter((name) => name === 'environment_create_download_directory').length };
  });
  expect(result.request.quality).toBe('sdr');
  expect(result.request.media_preferences.video).toEqual([]);
  expect(result.request.media_preferences.audio).toEqual(['30280']);
  expect(result.creates).toBe(0);
});


test('download tabs use saved presets and keep overrides local', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1100, height: 900 });
  await installTauriMock(page, 'light');
  await page.addInitScript(() => {
    const target = window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> }; __BDL_PRESET_REQUEST__?: unknown };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      const result = await invoke(command, args);
      if (command === 'settings_get') return { ...(result as object), naming_template: '{title}/{part_title}.{ext}', naming_presets: [{ id: 'favorites', name: '我的收藏', template: '{title}.{ext}' }] };
      if (command === 'selection_create_tasks') target.__BDL_PRESET_REQUEST__ = args?.request;
      return result;
    };
  });
  await page.goto('/');
  await page.getByLabel('链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();
  await page.getByRole('row', { name: /测试视频/ }).click();
  await page.getByRole('button', { name: '下载所选 (1)' }).click();
  const dialog = page.getByRole('dialog', { name: '下载设置' });
  await expect(dialog.getByRole('tab', { name: '常规', exact: true })).toHaveAttribute('aria-selected', 'true');
  await dialog.getByRole('combobox', { name: '命名预设', exact: true }).click();
  await page.getByRole('option', { name: '我的收藏', exact: true }).click();
  await dialog.getByRole('combobox', { name: /重名处理/ }).click();
  await page.getByRole('option', { name: '扩展文件名', exact: true }).click();
  await dialog.getByRole('button', { name: '取消', exact: true }).click();
  await page.getByRole('button', { name: '下载所选 (1)' }).click();
  await expect(dialog.getByRole('combobox', { name: /重名处理/ })).toContainText('已有文件则跳过');
  await expect(dialog.getByRole('textbox', { name: '命名模板', exact: true })).toHaveValue('{title}/{part_title}.{ext}');
  await dialog.getByRole('combobox', { name: '命名预设', exact: true }).click();
  await page.getByRole('option', { name: '我的收藏', exact: true }).click();
  await dialog.getByRole('combobox', { name: /重名处理/ }).click();
  await page.getByRole('option', { name: '扩展文件名', exact: true }).click();
  await expect(dialog.getByRole('combobox', { name: '命名预设', exact: true })).toContainText('我的收藏');
  await expect(page.getByRole('listbox')).toHaveCount(0);
  await page.screenshot({ path: testInfo.outputPath('download-general.png') });
  const generalBounds = await dialog.boundingBox();
  await dialog.getByRole('tab', { name: '画质与音频' }).click();
  await expect.poll(async () => (await dialog.boundingBox())?.height).toBe(generalBounds?.height);
  await expect.poll(async () => (await dialog.boundingBox())?.y).toBe(generalBounds?.y);
  await page.screenshot({ path: testInfo.outputPath('download-media.png') });
  await page.setViewportSize({ width: 900, height: 600 });
  await dialog.getByRole('tab', { name: '常规', exact: true }).click();
  const scrollArea = dialog.locator('.download-options-scroll');
  await expect.poll(() => scrollArea.evaluate((element) => element.scrollHeight > element.clientHeight)).toBe(true);
  await scrollArea.evaluate((element) => { element.scrollTop = element.scrollHeight; });
  await expect.poll(() => scrollArea.evaluate((element) => element.scrollTop)).toBeGreaterThan(0);
  await expect(dialog.getByRole('button', { name: '恢复默认偏好' })).toBeInViewport();
  await expect(dialog.getByRole('tab', { name: '常规', exact: true })).toBeInViewport();
  await page.screenshot({ path: testInfo.outputPath('download-scroll.png') });
  await dialog.getByRole('button', { name: '开始下载' }).click();
  const result = await page.evaluate(() => {
    const target = window as unknown as { __BDL_PRESET_REQUEST__: { naming_template: string; duplicate_naming_strategy: string }; __BDL_TEST_INVOKES__: string[] };
    return { request: target.__BDL_PRESET_REQUEST__, saves: target.__BDL_TEST_INVOKES__.filter((name) => name === 'settings_update').length };
  });
  expect(result.request.naming_template).toBe('{title}.{ext}');
  expect(result.request.duplicate_naming_strategy).toBe('append_suffix');
  expect(result.saves).toBe(0);
});


test('saved naming presets are reusable while unsaved defaults stay in settings', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1100, height: 900 });
  await installTauriMock(page, 'light');
  await page.addInitScript(() => {
    const target = window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> } };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => command === 'settings_update' ? args?.settings : invoke(command, args);
  });
  await page.goto('/');
  await page.getByRole('button', { name: '设置 偏好与维护' }).click();
  await page.getByRole('button', { name: /文件命名/ }).click();
  await page.getByRole('textbox', { name: '命名模板', exact: true }).fill('{owner_name}/{title}.{ext}');
  await page.getByRole('textbox', { name: '预设名称', exact: true }).fill('按 UP 收藏');
  await page.getByRole('button', { name: '保存为预设', exact: true }).click();
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByRole('button', { name: '保存', exact: true })).toBeDisabled();
  await page.screenshot({ path: testInfo.outputPath('naming-presets-settings.png') });
  await page.getByRole('textbox', { name: '命名模板', exact: true }).fill('{bvid}.{ext}');
  await page.getByRole('button', { name: '解析 添加与选择' }).click();
  await page.getByLabel('链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();
  await page.getByRole('row', { name: /测试视频/ }).click();
  await page.getByRole('button', { name: '下载所选 (1)' }).click();
  const dialog = page.getByRole('dialog', { name: '下载设置' });
  await expect(dialog.getByRole('combobox', { name: '命名预设', exact: true })).toContainText('按 UP 收藏');
  await expect(dialog.getByText('文件名预览：')).toContainText('示例UP/示例视频.mp4');
});


test('download archive settings inherit defaults and override processing per task', async ({ page }, testInfo) => {
  await installTauriMock(page, 'light');
  await page.addInitScript(() => {
    const target = window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> }; __BDL_ARCHIVE_REQUEST__?: unknown };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      const result = await invoke(command, args);
      if (command === 'settings_get') return { ...(result as object), output_extension: 'mkv', archive_mode: 'custom', retain_raw_streams: true, embed_cover: true, embed_subtitles: true };
      if (command === 'selection_create_tasks') target.__BDL_ARCHIVE_REQUEST__ = args?.request;
      return result;
    };
  });
  await page.goto('/');
  await page.getByLabel('链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();
  await page.getByRole('row', { name: /测试视频/ }).click();
  await page.getByRole('button', { name: '下载所选 (1)' }).click();
  const dialog = page.getByRole('dialog', { name: '下载设置' });
  await dialog.getByRole('tab', { name: '附加内容', exact: true }).click();
  for (const label of ['保留原始视频/音频轨道', '嵌入封面（仅 MKV）', '嵌入字幕（仅 MKV）']) {
    await expect(dialog.getByRole('checkbox', { name: label, exact: true })).toBeChecked();
    await dialog.getByRole('checkbox', { name: label, exact: true }).uncheck();
  }
  await page.screenshot({ path: testInfo.outputPath('download-archive.png') });
  await dialog.getByRole('button', { name: '开始下载' }).click();
  const result = await page.evaluate(() => {
    const target = window as unknown as { __BDL_ARCHIVE_REQUEST__: Record<string, unknown>; __BDL_TEST_INVOKES__: string[] };
    return { request: target.__BDL_ARCHIVE_REQUEST__, saves: target.__BDL_TEST_INVOKES__.filter((name) => name === 'settings_update').length };
  });
  expect(result.request).toMatchObject({ retain_raw_streams: false, embed_cover: false, embed_subtitles: false, archive_mode: 'custom', output_extension: 'mkv' });
  expect(result.saves).toBe(0);
});


test('pagination presets save rules and allow custom cooldown', async ({ page }, testInfo) => {
  await installTauriMock(page, 'light');
  await page.addInitScript(() => {
    const target = window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> }; __RULES__?: unknown };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      if (command === 'settings_update') { target.__RULES__ = (args?.settings as { parse_rules: unknown }).parse_rules; return args?.settings; }
      return invoke(command, args);
    };
  });
  await page.goto('/');
  await page.getByRole('button', { name: '设置 偏好与维护' }).click();
  await expect(page.getByRole('combobox', { name: '解析预设', exact: true })).toContainText('标准');
  await expect(page.getByText('解析约 200 条，额外等待约 5 秒，不含网络耗时。')).toBeVisible();
  await page.getByRole('button', { name: '为什么解析需要等待' }).hover();
  await expect(page.locator('[data-slot="text"]').filter({ hasText: 'B站可能限制连续请求' })).toBeVisible();
  await page.getByRole('heading', { name: '解析节奏' }).hover();
  await page.getByRole('combobox', { name: '解析预设', exact: true }).click();
  await page.getByRole('option', { name: '快速', exact: true }).click();
  await expect(page.getByRole('combobox', { name: '每批解析', exact: true })).toContainText('100 条');
  await page.getByRole('combobox', { name: '每100条休息', exact: true }).click();
  await page.getByRole('option', { name: '10 秒', exact: true }).click();
  await expect(page.getByRole('combobox', { name: '解析预设', exact: true })).toContainText('自定义');
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByRole('button', { name: '保存', exact: true })).toBeDisabled();
  expect(await page.evaluate(() => (window as unknown as { __RULES__: unknown }).__RULES__)).toEqual({ pages_per_round: 5, interval_seconds: 1, rest_seconds: 10 });
  await page.screenshot({ path: testInfo.outputPath('pagination-presets.png') });
});


for (const scenario of ['confirmed', 'first-run', 'save-failure'] as const) {
  test(`native preferences startup without browser preference storage: ${scenario}`, async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (error) => errors.push(error.message));
    await installTauriMock(page, 'dark');
    await page.addInitScript(({ scenario }) => {
      // Vite's Vue devtools use their own storage at import time. Deny all BDL
      // preferences while leaving development-only tooling outside this test.
      const getItem = Storage.prototype.getItem;
      const setItem = Storage.prototype.setItem;
      Storage.prototype.getItem = function (key) {
        if (key.startsWith('bdl.')) throw new DOMException('Storage denied', 'SecurityError');
        return getItem.call(this, key);
      };
      Storage.prototype.setItem = function (key, value) {
        if (key.startsWith('bdl.')) throw new DOMException('Storage denied', 'SecurityError');
        return setItem.call(this, key, value);
      };
      const target = window as unknown as {
        __TAURI_INTERNALS__: { invoke: (command: string, args?: { settings?: unknown }) => Promise<unknown> };
        __SAVED_PREFERENCES__?: unknown;
      };
      const invoke = target.__TAURI_INTERNALS__.invoke;
      target.__TAURI_INTERNALS__.invoke = async (command, args) => {
        if (command === 'settings_get') {
          return { ...await invoke(command) as object, usage_notice_acknowledged: scenario === 'confirmed' };
        }
        if (command === 'settings_update') {
          if (scenario === 'save-failure') throw new Error('Settings are read only');
          target.__SAVED_PREFERENCES__ = args?.settings;
          return args?.settings;
        }
        return invoke(command, args);
      };
    }, { scenario });
    await page.goto('/');
    await expect(page.locator('.nav-item')).toHaveCount(5);
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
    if (scenario !== 'confirmed') {
      await page.getByRole('button', { name: '我已阅读并了解，继续使用' }).click();
      await expect(page.getByRole('dialog', { name: '使用前请阅读' })).toBeHidden();
      if (scenario === 'first-run') {
        await expect.poll(() => page.evaluate(() => (window as unknown as { __SAVED_PREFERENCES__?: { usage_notice_acknowledged: boolean } }).__SAVED_PREFERENCES__?.usage_notice_acknowledged)).toBe(true);
      } else {
        await expect(page.getByText('设置保存失败：Settings are read only')).toBeVisible();
      }
    } else {
      await expect(page.getByRole('dialog', { name: '使用前请阅读' })).toHaveCount(0);
    }
    await expect(page.locator('.main-region')).toHaveAttribute('data-page', 'parse');
    expect(errors).toEqual([]);
  });
}
