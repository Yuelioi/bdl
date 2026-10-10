import { expect, test, type Page } from '@playwright/test';
import { readFile } from 'node:fs/promises';
import type { NormalizedSourceTree } from '../../src/shared/api/dto';
import type { DownloadTask } from '../../src/shared/api/dto';

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
            if (command === 'settings_update') return (args as unknown as { settings: unknown }).settings;
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

test.describe('safe file removal', () => {
  test.use({ deviceScaleFactor: 2.25 });
  for (const theme of ['light', 'dark'] as const) {
    for (const device of ['desktop', 'tablet', 'phone'] as const) {
      test(`${theme} ${device} previews explicit selection and confirms deletion`, async ({ page }, testInfo) => {
        await page.setViewportSize(device === 'desktop' ? { width: 1280, height: 800 } : device === 'tablet' ? { width: 568, height: 356 } : { width: 320, height: 844 });
        if (device !== 'desktop') await page.addInitScript(() => Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }));
        await installTauriMock(page, theme, true, true);
        await page.addInitScript(() => {
          const target = window as unknown as {
            __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
            __REMOVAL_CALLS__: Array<{ command: string; args?: Record<string, unknown> }>;
          };
          target.__REMOVAL_CALLS__ = [];
          let tasks = ['selected', 'unselected'].map((id) => ({
            id, title: id === 'selected' ? '待删除任务' : '需要保留的任务', source_id: 'video:fixture', status: 'completed', resources: [],
            output_path: `downloads/${'很长的中文下载目录/'.repeat(20)}${id}.mp4`, media_selection: null, scheduled_at: null, speed_limit_bytes_per_second: null,
          }));
          let selectedIds: string[] = [];
          let failedOnce = false;
          const invoke = target.__TAURI_INTERNALS__.invoke;
          target.__TAURI_INTERNALS__.invoke = async (command, args) => {
            target.__REMOVAL_CALLS__.push({ command, args });
            if (command === 'queue_list') return tasks;
            if (command === 'queue_logs') return [];
            if (command === 'queue_delete_preview') {
              selectedIds = (args?.request as { task_ids: string[] }).task_ids;
              return { token: 'fixture-confirmation', task_ids: selectedIds, files: tasks.filter((t) => selectedIds.includes(t.id)).map((t) => t.output_path), file_count: selectedIds.length, total_bytes: 8192, preserved: ['其他任务仍在使用：shared.mp4'], roots: [] };
            }
            if (command === 'queue_delete_files') {
              if (!failedOnce) { failedOnce = true; throw { code: 'cleanup_failed', message: '文件被占用，请关闭播放器后重新确认' }; }
              tasks = tasks.filter((t) => !selectedIds.includes(t.id));
              return { updated: [], removed: selectedIds, failed: [] };
            }
            if (command === 'maintenance_temp_preview') return { token: 'temp-confirmation', task_ids: [], files: ['downloads/orphan.video.m4s.bdlpart.seg0'], file_count: 1, total_bytes: 4096, preserved: ['仍有任务使用的文件'], roots: ['downloads'] };
            if (command === 'maintenance_cleanup_temp') return { removed_files: 1, path: 'downloads' };
            return invoke(command, args);
          };
        });
        await page.goto('/');
        await page.getByRole('button', { name: /传输/ }).click();
        await page.getByRole('tab', { name: /全部/ }).click();
        const row = page.getByRole('list', { name: '传输任务', includeHidden: true }).getByRole('listitem', { includeHidden: true }).filter({ hasText: '待删除任务' });
        await row.getByRole('button', { name: /更多操作/ }).click();
        await page.getByRole(device === 'desktop' ? 'menuitem' : 'button', { name: '删除任务及文件', exact: true }).click();
        const dialog = page.getByRole('dialog', { name: '删除任务及文件', exact: true });
        await expect(dialog.getByText(/1 个成品或附件/)).toBeVisible();
        const deletionCalls = () => page.evaluate(() => (window as unknown as { __REMOVAL_CALLS__: Array<{ command: string }> }).__REMOVAL_CALLS__.filter((c) => c.command === 'queue_delete_files').length);
        expect(await deletionCalls()).toBe(0);
        await dialog.getByRole('button', { name: '取消', exact: true }).click();
        await expect(dialog).toBeHidden(); expect(await deletionCalls()).toBe(0);
        if (device !== 'desktop') await page.getByRole('button', { name: '管理任务', exact: true }).click();
        await row.getByRole('checkbox').check();
        if (device !== 'desktop') {
          const toolbar = page.locator('.transfer-footer .bulk-actions');
          expect(await toolbar.evaluate((el) => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(1);
          for (const button of await toolbar.getByRole('button').all()) {
            expect(await button.evaluate((el) => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(1);
          }
        }
        await page.locator('.bulk-action-bar').getByRole('button', { name: '删除任务及文件', exact: true }).click();
        await expect(dialog.getByText(/1 个成品或附件/)).toBeVisible();
        await dialog.getByText('查看将删除的文件', { exact: true }).click();
        await expect(dialog.getByText(/selected.mp4/, { exact: false })).toBeVisible();
        expect(await dialog.evaluate((element) => element.scrollWidth <= element.clientWidth + 1)).toBe(true);
        const box = (await dialog.boundingBox())!;
        expect(box.x).toBeGreaterThanOrEqual(15); expect(box.y).toBeGreaterThanOrEqual(15);
        expect(box.x + box.width).toBeLessThanOrEqual(page.viewportSize()!.width - 15);
        expect(box.y + box.height).toBeLessThanOrEqual(page.viewportSize()!.height - 15);
        await page.screenshot({ path: testInfo.outputPath(`file-removal-${device}-${theme}.png`) });
        await dialog.getByRole('button', { name: '确认删除任务及文件', exact: true }).click();
        await expect(dialog.getByText(/文件被占用/)).toBeVisible();
        await expect(dialog.getByRole('button', { name: '确认删除任务及文件', exact: true })).toBeDisabled();
        await expect(row).toBeVisible();
        await dialog.getByRole('button', { name: '重新查看文件', exact: true }).click();
        await dialog.getByRole('button', { name: '确认删除任务及文件', exact: true }).click();
        await expect(dialog).toBeHidden(); await expect(row).toHaveCount(0);
        await expect(page.getByText('需要保留的任务', { exact: true })).toBeVisible();
        const previews = await page.evaluate(() => (window as unknown as { __REMOVAL_CALLS__: Array<{ command: string; args?: { request?: { task_ids: string[] } } }> }).__REMOVAL_CALLS__.filter((c) => c.command === 'queue_delete_preview').map((c) => c.args?.request?.task_ids));
        expect(previews.every((ids) => JSON.stringify(ids) === '["selected"]')).toBe(true);
        if (device === 'desktop') {
          await page.getByRole('button', { name: '设置 偏好与维护' }).click();
          await page.getByRole('button', { name: /网络与维护/ }).click();
        } else {
          await page.getByRole('navigation', { name: '主导航' }).getByRole('button', { name: '我的', exact: true }).click();
          await page.getByRole('button', { name: /清理与维护/ }).click();
        }
        await page.getByRole('button', { name: '清理临时文件', exact: true }).click();
        const cleanup = page.getByRole('dialog', { name: '清理临时文件', exact: true });
        await expect(cleanup.getByText(/1 个临时文件/)).toBeVisible();
        await cleanup.getByRole('button', { name: '取消', exact: true }).click();
        const tempCalls = () => page.evaluate(() => (window as unknown as { __REMOVAL_CALLS__: Array<{ command: string }> }).__REMOVAL_CALLS__.filter((c) => c.command === 'maintenance_cleanup_temp').length);
        expect(await tempCalls()).toBe(0);
        await page.getByRole('button', { name: '清理临时文件', exact: true }).click();
        await cleanup.getByRole('button', { name: '确认清理', exact: true }).click();
        await expect(cleanup).toBeHidden(); expect(await tempCalls()).toBe(1);
      });
    }
  }
});

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
  expect(await parseRow.locator('.result-copy strong').evaluate((title) => getComputedStyle(title).fontSize)).toBe(
    '13px',
  );
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

test.describe('adaptive Android workspace', () => {
  test.use({ deviceScaleFactor: 2.25 });
  for (const theme of ['light', 'dark'] as const) {
    test(`${theme} rail and compact details preserve selection on rotation`, async ({ page }, testInfo) => {
      await page.setViewportSize({ width: 568, height: 356 });
      await page.addInitScript(() =>
        Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Tablet' }),
      );
      if (theme === 'dark')
        await page.addInitScript(() => {
          const matchMedia = window.matchMedia.bind(window);
          window.matchMedia = (query) => {
            const media = matchMedia(query);
            if (query.includes('(min-width: 480px)') && query.includes('(max-height: 500px)')) {
              Object.defineProperties(media, {
                addEventListener: { value: undefined },
                removeEventListener: { value: undefined },
              });
            }
            return media;
          };
        });
      await installTauriMock(page, theme, true, true);
      await page.addInitScript(() => {
        const target = window as unknown as {
          __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> };
        };
        const invoke = target.__TAURI_INTERNALS__.invoke;
        target.__TAURI_INTERNALS__.invoke = async (command, args) => {
          const result = await invoke(command, args);
          if (command === 'parse_create_source') {
            const tree = result as NormalizedSourceTree;
            tree.source.title = '【免费开源】Rust 开发的新一代哔哩哔哩下载器 | 4k+批量（请勿滥用）';
            tree.groups[0]!.items[0]!.title = tree.source.title;
            tree.groups[0]!.items[0]!.parts[0]!.title = tree.source.title;
            const items = tree.groups[0]!.items;
            tree.groups[0]!.items = Array.from(
              { length: 40 },
              (_, index) =>
                items[index] ?? {
                  ...items[1]!,
                  id: `density-item-${index}`,
                  title: `分页视频 ${index + 1}`,
                  parts: [{ ...items[1]!.parts[0]!, id: `density-part-${index}`, title: `分页视频 ${index + 1}` }],
                },
            );
            tree.source.loaded_count = 40;
          }
          return result;
        };
      });
      await page.goto('/');
      await page.getByLabel('Bilibili 链接或 BV / AV').fill('favorite:fixture');
      await page.getByRole('button', { name: '开始解析' }).click();
      const results = page.getByRole('list', { name: '解析结果' });
      await expect(results.getByRole('checkbox')).toHaveCount(0);
      await page.getByRole('button', { name: '管理视频', exact: true }).click();
      await expect(results.getByRole('checkbox')).toHaveCount(40);
      await results.getByRole('listitem').first().locator('.mobile-result-content').click();
      const nav = page.getByRole('navigation', { name: '主导航' });
      const download = page.getByRole('button', { name: '下载所选 (1)', exact: true });
      await page.screenshot({ path: testInfo.outputPath(`android-${theme}-568x356.png`), animations: 'disabled' });
      expect((await nav.boundingBox())!.width).toBeLessThanOrEqual(568 * 0.12);
      expect((await results.getByRole('listitem').first().boundingBox())!.height).toBeCloseTo(64 * 0.6, 1);
      expect((await results.locator('.result-cover').first().boundingBox())!.height).toBeCloseTo(54 * 0.6, 1);
      expect((await results.locator('.result-cover').first().boundingBox())!.width).toBeCloseTo(96 * 0.6, 1);
      const resultCard = (await results.boundingBox())!;
      const firstRow = (await results.getByRole('listitem').first().boundingBox())!;
      expect(firstRow.x - resultCard.x).toBeGreaterThan(6);
      expect(firstRow.y - resultCard.y).toBeGreaterThan(3);
      expect((await results.getByRole('checkbox').first().boundingBox())!.height / firstRow.height).toBeLessThan(0.35);
      expect(
        await results
          .locator('.result-copy strong')
          .first()
          .evaluate((title) => parseFloat(getComputedStyle(title).fontSize)),
      ).toBeCloseTo(11 * 0.6, 1);
      expect((await nav.getByRole('button').first().boundingBox())!.height).toBeLessThanOrEqual(27);
      for (let index = 0; index < 7; index++)
        await expect(results.getByRole('listitem').nth(index)).toBeInViewport({ ratio: 1 });
      const checkboxBox = (await results.getByRole('checkbox').first().boundingBox())!;
      const coverBox = (await results.locator('.result-cover').first().boundingBox())!;
      expect(checkboxBox.x - firstRow.x).toBeGreaterThan(6);
      expect(coverBox.x - checkboxBox.x - checkboxBox.width).toBeGreaterThan(6);
      const searchButton = page.getByRole('button', { name: '搜索视频', exact: true });
      expect((await searchButton.boundingBox())!.width).toBeCloseTo(36 * 0.6, 1);
      expect((await searchButton.locator('svg').boundingBox())!.width).toBeCloseTo(20 * 0.6, 1);
      await searchButton.click();
      const filter = page.getByPlaceholder('筛选标题或作者');
      await filter.fill('no-match-fixture');
      await expect(page.getByText('没有匹配的视频', { exact: true })).toBeVisible();
      await filter.fill('分页视频 20');
      await expect(results.getByRole('listitem')).toHaveCount(1);
      await expect(download).toBeEnabled();
      await page.getByRole('checkbox', { name: '全选已加载', exact: true }).check();
      await expect(page.getByRole('button', { name: '下载所选 (40)', exact: true })).toBeEnabled();
      await page.getByRole('checkbox', { name: '全选已加载', exact: true }).uncheck();
      await page.screenshot({ path: testInfo.outputPath(`android-${theme}-filtered-card.png`) });
      await page.getByRole('button', { name: '关闭搜索', exact: true }).click();
      await expect(results.getByRole('listitem')).toHaveCount(40);
      await page.getByRole('button', { name: '完成管理', exact: true }).click();
      await expect(results.getByRole('checkbox')).toHaveCount(0);
      await expect(page.getByRole('button', { name: '下载所选 (0)', exact: true })).toBeDisabled();
      await page.getByRole('button', { name: '内容操作', exact: true }).click();
      const contentPanel = page.getByRole('dialog', { name: '内容操作' });
      await expect(contentPanel.locator('.tablet-action-body')).toBeVisible();
      const expectCenteredPanel = async () => {
        const viewport = page.viewportSize()!;
        await expect
          .poll(async () => {
            const box = (await contentPanel.boundingBox())!;
            return (
              Math.abs(box.x + box.width / 2 - viewport.width / 2) < 1 &&
              Math.abs(box.y + box.height / 2 - viewport.height / 2) < 1 &&
              box.width < viewport.width * 0.6 &&
              box.y > 0 &&
              box.y + box.height < viewport.height
            );
          })
          .toBe(true);
      };
      await expectCenteredPanel();
      await page.getByLabel('每次加载').selectOption('100');
      await page.screenshot({ path: testInfo.outputPath(`tablet-content-panel-${theme}.png`), animations: 'disabled' });
      await page.setViewportSize({ width: 390, height: 844 });
      await expect(contentPanel.locator('.mobile-sheet-body')).toBeVisible();
      await expect(page.getByLabel('每次加载')).toHaveValue('100');
      await expect
        .poll(async () => {
          const box = (await contentPanel.boundingBox())!;
          return [Math.round(box.x), Math.round(box.width), Math.round(box.y + box.height)];
        })
        .toEqual([0, 390, 844]);
      await page.screenshot({ path: testInfo.outputPath(`phone-content-panel-${theme}.png`), animations: 'disabled' });
      await page.setViewportSize({ width: 568, height: 356 });
      await expect(contentPanel.locator('.tablet-action-body')).toBeVisible();
      await expectCenteredPanel();
      await expect(page.getByLabel('每次加载')).toHaveValue('100');
      await page.keyboard.press('Escape');
      await expect(contentPanel).not.toBeVisible();
      await results.getByRole('listitem').first().locator('.mobile-result-content').click();
      await page.evaluate(() => {
        const style = document.documentElement.style;
        style.setProperty('--bdl-safe-area-top', '8px');
        style.setProperty('--bdl-safe-area-bottom', '12px');
        style.setProperty('--bdl-safe-area-left', '16px');
        style.setProperty('--bdl-safe-area-right', '20px');
      });
      for (const viewport of [
        { width: 568, height: 356, rail: true },
        { width: 768, height: 900, rail: true },
        { width: 767, height: 900, rail: false },
        { width: 600, height: 500, rail: true },
        { width: 600, height: 501, rail: false },
        { width: 480, height: 320, rail: true },
        { width: 479, height: 320, rail: false },
        { width: 1280, height: 800, rail: true },
        { width: 390, height: 844, rail: false },
        { width: 568, height: 356, rail: true },
      ]) {
        await page.setViewportSize({ width: viewport.width, height: viewport.height });
        await expect.poll(async () => (await nav.boundingBox())!.width <= 88).toBe(viewport.rail);
        const navigation = (await nav.boundingBox())!;
        const workspace = (await page.locator('.workspace-mobile').boundingBox())!;
        await expect(download).toBeEnabled();
        await expect(download).toBeInViewport({ ratio: 1 });
        for (const button of await nav.getByRole('button').all()) await expect(button).toBeInViewport({ ratio: 1 });
        if (viewport.rail) {
          expect(navigation.width).toBeLessThanOrEqual(88);
          expect(workspace.x).toBeGreaterThanOrEqual(navigation.x + navigation.width);
          expect(workspace.height).toBeGreaterThanOrEqual(viewport.height - 8);
          expect((await page.locator('.mobile-detail-copy h2').boundingBox())!.height).toBeLessThanOrEqual(24);
          expect(workspace.x + workspace.width).toBeLessThanOrEqual(viewport.width - 20);
          if (viewport.width === 568) {
            for (let index = 0; index < 6; index++)
              await expect(results.getByRole('listitem').nth(index)).toBeInViewport({ ratio: 1 });
          }
        } else {
          expect(navigation.y).toBeGreaterThanOrEqual(workspace.y + workspace.height);
          expect(navigation.width).toBe(viewport.width);
          expect((await results.getByRole('listitem').first().boundingBox())!.height).toBe(62);
        }
        expect(
          await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth),
        ).toBeLessThanOrEqual(0);
        if (viewport.width === 1280)
          await page.screenshot({ path: testInfo.outputPath(`android-${theme}-1280x800.png`), animations: 'disabled' });
      }
      await download.click();
      const dialog = page.getByRole('dialog', { name: '下载设置' });
      await expect(dialog.getByRole('button', { name: '开始下载', exact: true })).toBeInViewport({ ratio: 1 });
      const bounds = (await dialog.boundingBox())!;
      expect(bounds.y).toBeGreaterThanOrEqual(8);
      expect(bounds.y + bounds.height).toBeLessThanOrEqual(356 - 12);
      const preset = dialog.getByRole('combobox', { name: /^下载预设/ });
      const trigger = await preset.elementHandle();
      await preset.click();
      const option = page.getByRole('option', { name: '快速下载', exact: true });
      await expect(option).toBeInViewport({ ratio: 0.99 });
      const optionBox = (await option.boundingBox())!;
      expect(optionBox.y).toBeGreaterThanOrEqual(-1);
      expect(optionBox.y + optionBox.height).toBeLessThanOrEqual(page.viewportSize()!.height + 1);
      await expect
        .poll(async () => {
          const popup = (await page.getByRole('listbox').boundingBox())!;
          const anchor = (await trigger!.boundingBox())!;
          return {
            width: Math.abs(popup.width - anchor.width) <= 2,
            left: Math.abs(popup.x - anchor.x) <= 2,
            below: popup.y >= anchor.y + anchor.height - 1,
          };
        })
        .toEqual({ width: true, left: true, below: true });
      await option.click();
      await expect(preset).toContainText('快速下载');
      await page.screenshot({
        path: testInfo.outputPath(`android-${theme}-scaled-dialog.png`),
        animations: 'disabled',
      });
      await page.keyboard.press('Escape');
      await expect(download).toBeFocused();
      const libraryButton = nav.getByRole('button', { name: '内容库', exact: true });
      await libraryButton.focus();
      await page.keyboard.press('Enter');
      await expect(libraryButton).toHaveAttribute('aria-current', 'page');
      await expect(page.locator('.library-panel')).toBeVisible();
      await nav.getByRole('button', { name: '传输', exact: true }).click();
      await expect(page.locator('.transfer-main')).toBeVisible();
      await nav.getByRole('button', { name: '我的', exact: true }).click();
      await expect(page.locator('.profile-card')).toBeVisible();
      await page.screenshot({ path: testInfo.outputPath(`android-${theme}-personal.png`), animations: 'disabled' });
      await nav.getByRole('button', { name: '解析', exact: true }).click();
      await expect(download).toBeEnabled();
      const calls = await page.evaluate(
        () => (window as unknown as { __BDL_TEST_INVOKES__: string[] }).__BDL_TEST_INVOKES__,
      );
      expect(calls.filter((command) => command === 'parse_create_source')).toHaveLength(1);
      await expect(page.locator('.window-controls')).toHaveCount(0);
    });
  }
});

for (const theme of ['light', 'dark'] as const) {
  test(`tablet dialogs ${theme} stay within the viewport without translate or dvh`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width: 800, height: 1280 });
    await page.addInitScript(() => {
      Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Tablet' });
    });
    await installTauriMock(page, theme, true, true);
    await page.addInitScript(() => {
      const target = window as unknown as {
        __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> };
      };
      const invoke = target.__TAURI_INTERNALS__.invoke;
      target.__TAURI_INTERNALS__.invoke = async (command, args) => {
        const result = await invoke(command, args);
        if (command === 'parse_create_source')
          (result as NormalizedSourceTree).source.title = '这是一个特别长的收藏夹标题'.repeat(12);
        return result;
      };
    });
    await page.goto('/');
    // Simulate one unsupported CSS feature; this is not a complete old-WebView emulator.
    await page.addStyleTag({ content: '[role="dialog"] { translate: none !important; }' });
    await page.getByLabel('Bilibili 链接或 BV / AV').fill('favorite:fixture');
    await page.getByRole('button', { name: '开始解析' }).click();
    await page
      .getByRole('list', { name: '解析结果' })
      .getByRole('listitem')
      .first()
      .locator('.mobile-result-content')
      .click();
    await page.getByRole('button', { name: '下载所选 (1)' }).click();
    const dialog = page.getByRole('dialog', { name: '下载设置' });
    await expect(dialog).toBeVisible();
    // Drop declarations containing dvh just as a parser without dynamic viewport units would.
    const dropped = await page.evaluate(() => {
      let count = 0;
      const visit = (rules: CSSRuleList) => {
        for (const rule of Array.from(rules)) {
          if (rule instanceof CSSStyleRule) {
            for (const property of Array.from(rule.style)) {
              if (/\bdvh\b|\d+dvh/.test(rule.style.getPropertyValue(property))) {
                rule.style.removeProperty(property);
                count++;
              }
            }
          } else if ('cssRules' in rule) visit((rule as CSSGroupingRule).cssRules);
        }
      };
      for (const sheet of Array.from(document.styleSheets)) visit(sheet.cssRules);
      return count;
    });
    expect(dropped).toBeGreaterThan(0);
    await page.evaluate(() => {
      const style = document.documentElement.style;
      style.setProperty('--bdl-safe-area-top', '28px');
      style.setProperty('--bdl-safe-area-bottom', '24px');
      style.setProperty('--bdl-safe-area-left', '32px');
      style.setProperty('--bdl-safe-area-right', '20px');
    });
    const speed = dialog.getByRole('textbox', { name: /^单任务限速（MB\/s）/ });
    await dialog.getByRole('tab', { name: '调度', exact: true }).click();
    await speed.fill('3');
    for (const viewport of [
      { width: 800, height: 1280 },
      { width: 1280, height: 800 },
      { width: 600, height: 800 },
      { width: 640, height: 480 },
      { width: 375, height: 430 },
      { width: 320, height: 568 },
      { width: 800, height: 360 },
    ]) {
      await page.setViewportSize(viewport);
      await expect(speed).toHaveValue('3');
      await expect(dialog.getByRole('button', { name: '开始下载', exact: true })).toBeInViewport({ ratio: 1 });
      await dialog.getByRole('tab', { name: '下载', exact: true }).click();
      const scrollArea = dialog.locator('.download-options-scroll');
      await scrollArea.evaluate((element) => {
        element.scrollTop = element.scrollHeight;
      });
      await expect(dialog.getByRole('combobox', { name: /^重名处理/ })).toBeInViewport({ ratio: 1 });
      await page.screenshot({
        path: testInfo.outputPath(`tablet-dialog-${theme}-${viewport.width}x${viewport.height}.png`),
        animations: 'disabled',
      });
      const bounds = (await dialog.boundingBox())!;
      expect(bounds.x).toBeGreaterThanOrEqual(32);
      expect(bounds.y).toBeGreaterThanOrEqual(28);
      expect(bounds.x + bounds.width).toBeLessThanOrEqual(viewport.width - 20);
      expect(bounds.y + bounds.height).toBeLessThanOrEqual(viewport.height - 24);
      expect(Math.abs(bounds.x + bounds.width / 2 - (32 + viewport.width - 20) / 2)).toBeLessThanOrEqual(1);
      await dialog.getByRole('tab', { name: '调度', exact: true }).click();
    }
    await page.setViewportSize({ width: 800, height: 1280 });
    await dialog.getByRole('tab', { name: '下载', exact: true }).click();
    await dialog.getByRole('combobox', { name: /^重名处理/ }).click();
    await page.getByRole('option', { name: '扩展文件名', exact: true }).click();
    await expect(dialog.getByRole('combobox', { name: /^重名处理/ })).toContainText('扩展文件名');
    await page.keyboard.press('Escape');
    await expect(dialog).not.toBeVisible();
    await expect(page.getByRole('button', { name: '下载所选 (1)' })).toBeFocused();
  });
}

for (const theme of ['light', 'dark'] as const) {
  test(`tablet dialogs ${theme} preset editor keeps controls and draft on rotation`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width: 800, height: 1280 });
    await page.addInitScript(() =>
      Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Tablet' }),
    );
    await installTauriMock(page, theme, true, true);
    await installPresetPersistence(page);
    await page.goto('/');
    await openPresetSettings(page);
    const trigger = page.getByRole('button', { name: '编辑', exact: true });
    await trigger.click();
    const editor = page.getByRole('dialog', { name: '编辑预设', exact: true });
    await editor.getByRole('textbox', { name: '预设名称', exact: true }).fill('平板自定义下载预设');
    for (const viewport of [
      { width: 1280, height: 800 },
      { width: 500, height: 700 },
      { width: 800, height: 360 },
    ]) {
      await page.setViewportSize(viewport);
      await expect(editor.getByRole('textbox', { name: '预设名称', exact: true })).toHaveValue('平板自定义下载预设');
      await expect(editor.getByRole('button', { name: '保存预设', exact: true })).toBeInViewport({ ratio: 1 });
      await page.screenshot({
        path: testInfo.outputPath(`tablet-preset-${theme}-${viewport.width}x${viewport.height}.png`),
        animations: 'disabled',
      });
      const bounds = (await editor.boundingBox())!;
      expect(bounds.x).toBeGreaterThanOrEqual(16);
      expect(bounds.y).toBeGreaterThanOrEqual(16);
      expect(bounds.x + bounds.width).toBeLessThanOrEqual(viewport.width - 16);
      expect(bounds.y + bounds.height).toBeLessThanOrEqual(viewport.height - 16);
    }
    await editor.getByRole('button', { name: '取消', exact: true }).click();
    await expect(editor).not.toBeVisible();
    await expect(trigger).toBeFocused();
  });
}

test('mobile library uses compact covers and shared page actions', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.addInitScript(() =>
    Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }),
  );
  await installTauriMock(page, 'light', true, true);
  await page.addInitScript(() => {
    const target = window as unknown as {
      __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> };
    };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      if (command === 'settings_update') return (args as { settings: unknown }).settings;
      const result = await invoke(command, args);
      if (command === 'parse_create_source') {
        const tree = result as NormalizedSourceTree;
        tree.groups?.forEach((group) =>
          group.items.forEach((item, index) => {
            item.cover_url = index === 0 ? 'http://i0.hdslb.com/test-cover.svg' : null;
          }),
        );
      }
      return result;
    };
  });
  const referers: Array<string | undefined> = [];
  await page.route('https://i0.hdslb.com/test-cover.svg', async (route) => {
    referers.push(route.request().headers().referer);
    await route.fulfill({
      contentType: 'image/svg+xml',
      body: '<svg xmlns="http://www.w3.org/2000/svg" width="160" height="100"><rect width="160" height="100" fill="#c7d8cf"/></svg>',
    });
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
  expect(await folderRow.locator('.result-cover').evaluate((artwork) => getComputedStyle(artwork).borderRadius)).toBe(
    '4px',
  );
  await expect(folderRow.getByRole('checkbox')).toHaveCount(0);
  await page.getByRole('button', { name: '管理视频', exact: true }).click();
  await expect(folderRow.getByRole('checkbox')).toBeVisible();
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
  await page.getByRole('button', { name: '下载 目录、速度与任务恢复' }).click();
  await page.getByLabel('全局下载限速（MB/s）', { exact: true }).fill('10');
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

test('mobile library collection index has a padded group and two-line rows in dark theme', async ({
  page,
}, testInfo) => {
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
  const group = page.getByLabel('内容集合', { exact: true });
  expect(await group.evaluate((element) => parseFloat(getComputedStyle(element).borderRadius))).toBeGreaterThan(6);
  const groupBox = (await group.boundingBox())!;
  const firstCardBox = (await cards.first().boundingBox())!;
  const coverBox = (await cards.first().locator('.library-cover').boundingBox())!;
  expect(firstCardBox.x - groupBox.x).toBeGreaterThan(6);
  expect(coverBox.x - firstCardBox.x).toBeGreaterThan(6);
  await expect(cards.locator('.library-description')).toHaveCount(0);
  for (const card of await cards.all()) {
    expect((await card.boundingBox())!.height).toBeLessThanOrEqual(64);
    await expect(card.locator('.library-card-copy').locator(':scope > *')).toHaveCount(2);
    expect(await card.evaluate((element) => getComputedStyle(element).backgroundColor)).toBe('rgba(0, 0, 0, 0)');
  }
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(320);
  await page.screenshot({ path: testInfo.outputPath('mobile-library-index-dark.png') });
});

test.describe('populated Android density', () => {
  test.use({ deviceScaleFactor: 2.25 });
  for (const theme of ['light', 'dark'] as const) {
    test(`tablet library groups have row padding and shared search in ${theme}`, async ({ page }, testInfo) => {
      await page.setViewportSize({ width: 568, height: 356 });
      await page.addInitScript(() =>
        Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Tablet' }),
      );
      await installTauriMock(page, theme, true, true);
      await page.addInitScript(() => {
        const target = window as unknown as {
          __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> };
        };
        const invoke = target.__TAURI_INTERNALS__.invoke;
        target.__TAURI_INTERNALS__.invoke = async (command, args) => {
          const result = await invoke(command, args);
          if (command === 'account_library_list') {
            const list = result as { items: Array<{ media_id: string; title: string }>; total: number };
            const first = list.items[0]!;
            list.items = Array.from({ length: 8 }, (_, index) => ({
              ...first,
              media_id: `padded-${index}`,
              title: `页夹 ${index + 1} · 长标题测试长标题测试长标题测试`,
            }));
            list.total = 8;
          }
          return result;
        };
      });
      await page.goto('/');
      await page.getByRole('navigation', { name: '主导航' }).getByRole('button', { name: '内容库' }).click();
      for (const category of ['收藏夹', '订阅合集']) {
        await page.getByRole('tab', { name: new RegExp(category) }).click();
        const group = page.getByLabel('内容集合', { exact: true });
        const cards = group.getByRole('button');
        await expect(cards).toHaveCount(8);
        const first = (await cards.first().boundingBox())!;
        const second = (await cards.nth(1).boundingBox())!;
        const cover = (await cards.first().locator('.library-cover').boundingBox())!;
        expect(first.y).toBe(second.y);
        expect(second.x - first.x - first.width).toBeGreaterThan(6);
        expect(cover.x - first.x).toBeGreaterThan(6);
        expect(first.x - (await group.boundingBox())!.x).toBeGreaterThan(6);
        expect(await group.evaluate((element) => getComputedStyle(element).backgroundColor)).not.toBe(
          'rgba(0, 0, 0, 0)',
        );
        await page.screenshot({ path: testInfo.outputPath(`tablet-library-${theme}-${category}.png`) });
      }
      await page.getByRole('button', { name: '搜索内容库', exact: true }).click();
      await page.getByPlaceholder('搜索标题或创建者').fill('页夹 2');
      await expect(page.locator('.library-card')).toHaveCount(1);
      await page.getByRole('button', { name: '关闭搜索', exact: true }).click();
      await expect(page.locator('.library-card')).toHaveCount(8);
      await page.setViewportSize({ width: 320, height: 844 });
      expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(320);
      const cards = page.locator('.library-card');
      expect((await cards.nth(1).boundingBox())!.y).toBeGreaterThan((await cards.first().boundingBox())!.y);
      await cards.last().scrollIntoViewIfNeeded();
      await expect(cards.last()).toBeInViewport({ ratio: 1 });
      await cards.first().focus();
      await page.keyboard.press('Enter');
      await expect(page.locator('.folder-heading')).toContainText('页夹 1');
    });
    test(`single completed card has breathing room on tablet in ${theme}`, async ({ page }, testInfo) => {
      await page.setViewportSize({ width: 568, height: 356 });
      await page.addInitScript(() =>
        Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Tablet' }),
      );
      await installTauriMock(page, theme, true, true);
      await page.addInitScript(() => {
        const target = window as unknown as {
          __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> };
        };
        const invoke = target.__TAURI_INTERNALS__.invoke;
        target.__TAURI_INTERNALS__.invoke = async (command, args) => {
          if (command === 'queue_logs') return [];
          if (command === 'queue_list')
            return [
              {
                id: 'completed-card',
                title: '【Glyphshift】宣传片',
                source_id: 'video:fixture',
                status: 'completed',
                resources: [],
                output_path: 'downloads/Glyphshift.mp4',
                refresh_intent: { input: { kind: 'video_bvid', bvid: 'BV1fixture' }, cid: 1, duration_seconds: 53 },
                media_selection: null,
                scheduled_at: null,
                speed_limit_bytes_per_second: null,
              },
            ];
          return invoke(command, args);
        };
      });
      await page.goto('/');
      const nav = page.getByRole('navigation', { name: '主导航' });
      await nav.getByRole('button', { name: /传输/ }).click();
      await page.getByRole('tab', { name: /已完成/ }).click();
      const card = page.getByRole('list', { name: '传输任务' });
      const row = card.getByRole('listitem');
      await expect(page.getByRole('alert')).toHaveCount(0);
      await expect(row).toHaveCount(1);
      await expect(row).toContainText('0:53');
      const cardBox = (await card.boundingBox())!;
      const rowBox = (await row.boundingBox())!;
      expect(rowBox.x - cardBox.x).toBeGreaterThan(6);
      expect(rowBox.y - cardBox.y).toBeGreaterThan(3);
      expect(cardBox.height - rowBox.height).toBeGreaterThan(6);
      expect(cardBox.height - rowBox.height).toBeLessThan(16);
      expect(await card.evaluate((list) => parseFloat(getComputedStyle(list).borderRadius))).toBeGreaterThan(6);
      expect(
        await nav
          .getByRole('button', { name: '内容库' })
          .evaluate((button) => parseFloat(getComputedStyle(button).fontSize)),
      ).toBeCloseTo(9 * 0.6, 1);
      await page.screenshot({ path: testInfo.outputPath(`completed-card-${theme}.png`) });
      await row.getByRole('button', { name: /更多操作/ }).click();
      await expect(page.getByRole('dialog', { name: '任务操作' })).toBeVisible();
      await page.keyboard.press('Escape');
      await page.setViewportSize({ width: 390, height: 844 });
      await expect.poll(async () => (await row.boundingBox())!.height).toBe(62);
      expect(
        await nav
          .getByRole('button', { name: '内容库' })
          .evaluate((button) => parseFloat(getComputedStyle(button).fontSize)),
      ).toBe(11);
      await page.getByRole('button', { name: '管理任务' }).click();
      await row.getByRole('checkbox').check();
      const footer = page.locator('.transfer-footer');
      await expect(footer).toBeInViewport();
      expect((await footer.boundingBox())!.y).toBeGreaterThan(700);
      await page.screenshot({ path: testInfo.outputPath(`completed-card-phone-${theme}.png`) });
    });
  }

  for (const theme of ['light', 'dark'] as const) {
    test(`Android settings cards and new preset keep spacing in ${theme}`, async ({ page }, testInfo) => {
      await page.setViewportSize({ width: 568, height: 356 });
      await page.addInitScript(() =>
        Object.defineProperty(navigator, 'userAgent', {
          value: 'Mozilla/5.0 (Linux; Android 12) Tablet',
        }),
      );
      await installTauriMock(page, theme, true, true);
      await installPresetPersistence(page);
      await page.goto('/');
      await openPresetSettings(page);
      await page.getByRole('button', { name: '新增', exact: true }).click();
      const editor = page.getByRole('dialog', { name: '新增预设', exact: true });
      await expect(editor.getByRole('textbox', { name: '预设名称', exact: true })).toBeVisible();
      await page.screenshot({
        path: testInfo.outputPath(`settings-preset-content-${theme}.png`),
        animations: 'disabled',
      });
      const metrics = await editor.getByRole('tablist').evaluate((list) => ({
        clientWidth: list.clientWidth,
        scrollWidth: list.scrollWidth,
        clientHeight: list.clientHeight,
        scrollHeight: list.scrollHeight,
        marginBottom: getComputedStyle(list).marginBottom,
        overflowY: getComputedStyle(list).overflowY,
        triggers: Array.from(list.children).map((child) => ({
          state: child.getAttribute('data-state'),
          fontWeight: getComputedStyle(child).fontWeight,
          indicatorBottom: getComputedStyle(child, '::after').bottom,
        })),
      }));
      await testInfo.attach('preset-tab-metrics', { body: JSON.stringify(metrics), contentType: 'application/json' });
      const header = (await editor.locator('[data-slot="header"]').boundingBox())!;
      const label = (await editor.locator('.ui-form-field-label').first().boundingBox())!;
      expect(label.y - header.y - header.height).toBeGreaterThanOrEqual(6);
      expect(metrics.marginBottom).toBe('0px');
      expect(metrics.scrollHeight).toBeLessThanOrEqual(metrics.clientHeight);
      expect(metrics.scrollWidth).toBeLessThanOrEqual(metrics.clientWidth);
      const active = metrics.triggers.find((trigger) => trigger.state === 'active')!;
      const inactive = metrics.triggers.find((trigger) => trigger.state === 'inactive')!;
      expect(Number(active.fontWeight)).toBeGreaterThanOrEqual(600);
      expect(Number(active.fontWeight)).toBeGreaterThan(Number(inactive.fontWeight));
      expect(parseFloat(active.indicatorBottom)).toBeGreaterThanOrEqual(0);
      const selectedTab = editor.getByRole('tab', { selected: true });
      const selectedColor = await selectedTab.evaluate((tab) => getComputedStyle(tab).color);
      await selectedTab.hover();
      expect(await selectedTab.evaluate((tab) => getComputedStyle(tab).color)).toBe(selectedColor);
      const tabBounds = await editor.getByRole('tab').evaluateAll((tabs) =>
        tabs.map((tab) => {
          const box = tab.getBoundingClientRect();
          return { width: box.width, right: box.right };
        }),
      );
      expect(
        Math.max(...tabBounds.map((tab) => tab.width)) - Math.min(...tabBounds.map((tab) => tab.width)),
      ).toBeLessThan(1);
      const contentNote = (await editor.locator('.workflow-tab-content > p').first().boundingBox())!;
      const tabList = (await editor.getByRole('tablist').boundingBox())!;
      expect(contentNote.y - tabList.y - tabList.height).toBeGreaterThanOrEqual(6);
      await editor.getByRole('textbox', { name: '预设名称', exact: true }).fill('新增预设布局检查');
      await editor.getByRole('tab', { name: '内容', exact: true }).focus();
      await page.keyboard.press('ArrowRight');
      await expect(editor.getByRole('tab', { name: '转换', exact: true })).toHaveAttribute('aria-selected', 'true');
      await expect(editor.getByRole('region', { name: '视频', exact: true })).toBeVisible();
      for (const viewport of [
        { width: 390, height: 844 },
        { width: 320, height: 280 },
        { width: 568, height: 356 },
      ]) {
        await page.setViewportSize(viewport);
        await expect(editor.getByRole('textbox', { name: '预设名称', exact: true })).toHaveValue('新增预设布局检查');
        await expect(editor.getByRole('button', { name: '保存预设', exact: true })).toBeInViewport({ ratio: 0.99 });
        expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(viewport.width);
        if (viewport.width === 390)
          await page.screenshot({
            path: testInfo.outputPath(`settings-preset-phone-${theme}.png`),
            animations: 'disabled',
          });
      }
      await editor.getByRole('button', { name: '取消', exact: true }).click();
      await page.getByRole('button', { name: '返回设置', exact: true }).click();
      const categories = page.locator('.settings-category');
      await expect(categories).toHaveCount(5);
      const firstCategory = (await categories.first().boundingBox())!;
      const nextCategory = (await categories.nth(1).boundingBox())!;
      expect(nextCategory.y - firstCategory.y - firstCategory.height).toBeGreaterThanOrEqual(6);
      await page.screenshot({ path: testInfo.outputPath(`settings-index-cards-${theme}.png`), animations: 'disabled' });
      for (const [label, cardTitles] of [
        [/^下载 目录/, ['保存位置', '下载与恢复', '解析节奏']],
        [/^下载预设 内容/, ['默认预设', '质量选择']],
        [/^文件命名 模板/, ['命名模板', '命名预设', '重名处理']],
        [/^网络与维护/, ['网络与日志', '清理与诊断']],
      ] as const) {
        await page.getByRole('button', { name: label }).click();
        const cards = page.locator('.settings-content .settings-card');
        await expect(cards).toHaveCount(cardTitles.length);
        for (const title of cardTitles)
          await expect(page.getByRole('region', { name: title, exact: true })).toHaveCount(1);
        const styles = await cards.evaluateAll((items) =>
          items.map((item) => {
            const style = getComputedStyle(item);
            return {
              radius: parseFloat(style.borderRadius),
              padding: parseFloat(style.paddingLeft),
              background: style.backgroundColor,
            };
          }),
        );
        for (const style of styles) {
          expect(style.radius).toBeGreaterThanOrEqual(6);
          expect(style.padding).toBeGreaterThanOrEqual(8);
          expect(style.background).not.toBe('rgba(0, 0, 0, 0)');
        }
        const lastCard = cards.last();
        await lastCard.scrollIntoViewIfNeeded();
        await expect(lastCard).toBeInViewport();
        await page.locator('.settings-content').evaluate((element) => {
          element.scrollTop = 0;
        });
        await page.screenshot({
          path: testInfo.outputPath(`settings-function-${cardTitles[0]}-${theme}.png`),
          animations: 'disabled',
        });
        await page.getByRole('button', { name: '返回设置', exact: true }).click();
      }
    });

    test(`clean Android parse uses a centered three-row tablet form and a wide phone input in ${theme}`, async ({
      page,
    }, testInfo) => {
      await page.setViewportSize({ width: 568, height: 356 });
      await page.addInitScript(() =>
        Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Tablet' }),
      );
      await installTauriMock(page, theme, true, true);
      await page.goto('/');
      const form = page.locator('.mobile-parse-form');
      const input = page.getByLabel('Bilibili 链接或 BV / AV', { exact: true });
      await expect(input).toBeVisible();
      await expect(page.locator('.parse-intro, .parse-help')).toHaveCount(0);
      await input.fill('https://www.bilibili.com/video/av117330034104467/');
      for (const viewport of [
        { width: 568, height: 356 },
        { width: 1280, height: 800 },
      ]) {
        await page.setViewportSize(viewport);
        await expect
          .poll(() =>
            form.evaluate((element) => {
              const box = element.getBoundingClientRect();
              const workspace = element.closest('.mobile-parse-page')!.getBoundingClientRect();
              return (
                box.width / workspace.width > 0.7 &&
                box.width / workspace.width < 0.8 &&
                Math.abs(box.x + box.width / 2 - workspace.x - workspace.width / 2) < 1 &&
                Math.abs(box.y + box.height / 2 - workspace.y - workspace.height / 2) < 1
              );
            }),
          )
          .toBe(true);
        const box = (await form.boundingBox())!;
        const workspace = (await page.locator('.mobile-parse-page').boundingBox())!;
        expect(box.width / workspace.width).toBeGreaterThan(0.7);
        expect(box.width / workspace.width).toBeLessThan(0.8);
        expect(Math.abs(box.x + box.width / 2 - workspace.x - workspace.width / 2)).toBeLessThan(1);
        expect(Math.abs(box.y + box.height / 2 - workspace.y - workspace.height / 2)).toBeLessThan(1);
        const inputBox = (await input.boundingBox())!;
        const paste = page.getByRole('button', { name: '粘贴', exact: true });
        const range = page.getByLabel('解析范围', { exact: true });
        const submit = page.getByRole('button', { name: '开始解析', exact: true });
        const pasteBox = (await paste.boundingBox())!;
        const rangeBox = (await range.boundingBox())!;
        const submitBox = (await submit.boundingBox())!;
        expect(inputBox.y).toBeGreaterThan(pasteBox.y + pasteBox.height);
        expect(Math.abs(pasteBox.y - rangeBox.y)).toBeLessThan(1);
        expect(submitBox.y).toBeGreaterThan(inputBox.y + inputBox.height);
        expect(submitBox.width).toBeCloseTo(inputBox.width, 0);
        for (const control of [input, paste, range, submit]) {
          const controlBox = (await control.boundingBox())!;
          expect(controlBox.x + controlBox.width).toBeLessThanOrEqual(viewport.width);
        }
        await page.screenshot({ path: testInfo.outputPath(`clean-home-${viewport.width}-${theme}.png`) });
      }
      for (const width of [390, 320]) {
        await page.setViewportSize({ width, height: 844 });
        const box = (await input.boundingBox())!;
        expect(box.width / width).toBeGreaterThan(0.9);
        expect(box.height).toBeGreaterThanOrEqual(120);
        await expect(input).toHaveValue('https://www.bilibili.com/video/av117330034104467/');
        expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(width);
        await page.screenshot({ path: testInfo.outputPath(`clean-home-${width}-${theme}.png`) });
      }
      await page.setViewportSize({ width: 320, height: 280 });
      const shortWorkspace = (await page.locator('.mobile-parse-page').boundingBox())!;
      expect((await input.boundingBox())!.y).toBeGreaterThanOrEqual(shortWorkspace.y);
      const shortSubmit = page.getByRole('button', { name: '开始解析', exact: true });
      await shortSubmit.scrollIntoViewIfNeeded();
      const shortSubmitBox = (await shortSubmit.boundingBox())!;
      expect(shortSubmitBox.y + shortSubmitBox.height).toBeLessThanOrEqual(
        shortWorkspace.y + shortWorkspace.height + 1,
      );
      await page.setViewportSize({ width: 320, height: 844 });
      await page.getByLabel('解析范围', { exact: true }).selectOption('single');
      const single = page.getByLabel('视频链接或 BV / AV', { exact: true });
      await single.fill('BV1xx411c7mD');
      await single.focus();
      await page.keyboard.press('Tab');
      await expect(page.getByRole('button', { name: '开始解析', exact: true })).toBeFocused();
      await page.getByRole('button', { name: '开始解析', exact: true }).click();
      await expect(page.locator('.mobile-result')).toHaveCount(1);
    });
  }

  test('Android failed task offers a visible retry action in its menu', async ({ page }, testInfo) => {
    await page.setViewportSize({ width: 568, height: 356 });
    await page.addInitScript(() =>
      Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Tablet' }),
    );
    await installTauriMock(page, 'light', true, true);
    await page.addInitScript(() => {
      const target = window as unknown as {
        __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> };
        __RETRY_REQUESTS__: unknown[];
      };
      const invoke = target.__TAURI_INTERNALS__.invoke;
      const requests: unknown[] = [];
      target.__RETRY_REQUESTS__ = requests;
      const task = {
        id: 'failed-media',
        title: '中秋家传一味',
        source_id: 'video:fixture',
        status: 'failed',
        resources: [],
        output_path: 'downloads/中秋.mp4',
        refresh_intent: null,
        media_selection: null,
        scheduled_at: null,
        speed_limit_bytes_per_second: null,
      };
      target.__TAURI_INTERNALS__.invoke = async (command, args) => {
        if (command === 'queue_logs') return [];
        if (command === 'queue_list') return [task];
        if (command === 'queue_retry') {
          requests.push(args);
          task.status = 'waiting';
          return task;
        }
        return invoke(command, args);
      };
    });
    await page.goto('/');
    await page.getByRole('navigation', { name: '主导航' }).getByRole('button', { name: /传输/ }).click();
    await page.getByRole('tab', { name: /失败/ }).click();
    const row = page.locator('.mobile-task');
    await expect(row).toHaveCount(1);
    await row.getByRole('button', { name: /更多操作/ }).click();
    const menu = page.getByRole('dialog', { name: '任务操作' });
    const retry = menu.getByRole('button', { name: '重试', exact: true });
    await expect(retry).toBeVisible();
    await expect(retry).toBeEnabled();
    await page.screenshot({ path: testInfo.outputPath('failed-retry-menu.png') });
    await retry.click();
    await expect(menu).toHaveCount(0);
    await expect
      .poll(() => page.evaluate(() => (window as unknown as { __RETRY_REQUESTS__: unknown[] }).__RETRY_REQUESTS__))
      .toEqual([{ taskId: 'failed-media' }]);
    await expect(row).toHaveCount(0);
    await expect(page.getByRole('alert')).toHaveCount(0);
  });

  test('mobile redesign keeps content first with populated lists and bottom sheets', async ({ page }, testInfo) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.addInitScript(() =>
      Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }),
    );
    await installTauriMock(page, 'light', true, true);
    await page.addInitScript(() => {
      const target = window as unknown as {
        __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> };
      };
      const invoke = target.__TAURI_INTERNALS__.invoke;
      target.__TAURI_INTERNALS__.invoke = async (command, args) => {
        if (command === 'settings_update') return (args as { settings: unknown }).settings;
        if (command === 'queue_logs') return [];
        if (command === 'queue_list')
          return Array.from({ length: 243 }, (_, index) => ({
            id: `mobile-task-${index}`,
            title: ['测试屏幕 · 壁纸级 4K 8K HDR 画面', 'bilibili 8K 带你看亚洲 33 个国家', '异环高效锄地路线 1'][
              index % 3
            ],
            source_id: 'video:fixture',
            status: index === 0 ? 'downloading' : index === 1 ? 'paused' : index === 2 ? 'failed' : 'completed',
            resources:
              index === 0
                ? [
                    {
                      id: 'resource:video',
                      kind: 'video',
                      intent: 'video',
                      current_urls: [],
                      headers: [],
                      status: 'downloading',
                      target_path: 'downloads/video.m4s',
                      temp_path: 'downloads/video.m4s.part',
                    },
                  ]
                : [],
            output_path: 'downloads/测试视频.mp4',
            refresh_intent: {
              input: { kind: 'video_bvid', bvid: 'BV1fixture' },
              cid: index + 1,
              cover_url: 'http://i0.hdslb.com/transfer-cover.svg',
              duration_seconds: 60,
            },
            media_selection: null,
            scheduled_at: null,
            speed_limit_bytes_per_second: null,
          }));
        const result = await invoke(command, args);
        if (command === 'account_library_list') {
          const list = result as {
            items: Array<{ title: string; source_url: string; media_count: number }>;
            total: number;
          };
          list.total = 33;
          list.items[0] = { ...list.items[0], title: '默认收藏夹', source_url: 'favorite:fixture', media_count: 356 };
        }
        if (command === 'parse_create_source') {
          const tree = result as NormalizedSourceTree;
          tree.source.title = '默认收藏夹';
          tree.source.total_count = 356;
          tree.source.loaded_count = 40;
          const base = tree.groups[0].items[0];
          tree.groups[0].items = Array.from({ length: 40 }, (_, index) => ({
            ...base,
            id: `item-${index}`,
            title: ['测试屏幕 · 壁纸级 4K 8K HDR 画面', 'bilibili 8K 带你看亚洲 33 个国家', '异环高效锄地路线 1'][
              index % 3
            ],
            parts: [{ ...base.parts[0], id: `part-${index}` }],
          }));
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
    await page.setViewportSize({ width: 568, height: 356 });
    await expect.poll(async () => (await firstCollectionCover.boundingBox())!.height).toBeCloseTo(54 * 0.6, 1);
    expect((await firstCollection.boundingBox())!.height).toBeLessThanOrEqual(42);
    await page.screenshot({ path: testInfo.outputPath('redesign-library-compact.png') });
    await page.setViewportSize({ width: 390, height: 844 });
    await expect.poll(async () => (await firstCollectionCover.boundingBox())!.height).toBe(48);
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
    await expect(page.locator('.mobile-task').first()).toContainText('失败');
    await expect(page.locator('.mobile-task').filter({ hasText: '下载视频中' })).toHaveCount(1);
    expect((await page.locator('.mobile-task').first().boundingBox())!.height).toBeLessThanOrEqual(64);
    expect((await page.locator('.task-artwork').first().boundingBox())!.height).toBeLessThanOrEqual(49);
    expect(
      await page
        .locator('.task-artwork')
        .first()
        .evaluate((artwork) => getComputedStyle(artwork).borderRadius),
    ).toBe('4px');
    expect(
      await page
        .locator('.task-title')
        .first()
        .evaluate((title) => getComputedStyle(title).fontSize),
    ).toBe('13px');
    expect(
      await page
        .locator('.task-playback-action')
        .first()
        .locator('svg')
        .evaluate((icon) => icon.childElementCount),
    ).toBeGreaterThan(0);
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
    const fullyVisibleTaskCount = await page.locator('.mobile-task').evaluateAll(
      (tasks, bounds) =>
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
      for (const text of await tab.locator('span').all()) {
        const textBox = (await text.boundingBox())!;
        expect(textBox.x).toBeGreaterThanOrEqual(box!.x);
        expect(textBox.x + textBox.width).toBeLessThanOrEqual(box!.x + box!.width);
      }
    }
    const header = page.locator('.transfer-toolbar');
    const tabList = header.getByRole('tablist');
    const actions = header.locator('.mobile-list-actions');
    const tabBounds = (await tabList.boundingBox())!;
    const actionBounds = (await actions.boundingBox())!;
    expect(Math.abs(tabBounds.y + tabBounds.height / 2 - actionBounds.y - actionBounds.height / 2)).toBeLessThan(1);
    expect(tabBounds.x + tabBounds.width).toBeLessThanOrEqual(actionBounds.x);
    const tabMetrics = await tabList.evaluate((element) => ({
      clientWidth: element.clientWidth,
      scrollWidth: element.scrollWidth,
      clientHeight: element.clientHeight,
      scrollHeight: element.scrollHeight,
    }));
    expect(tabMetrics.scrollWidth).toBeLessThanOrEqual(tabMetrics.clientWidth);
    expect(tabMetrics.scrollHeight).toBeLessThanOrEqual(tabMetrics.clientHeight);
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
    await page
      .getByRole('button', { name: /：更多操作/ })
      .first()
      .click();
    await expect(page.getByRole('dialog', { name: '任务操作' })).toBeVisible();
    await expectSheetInViewport('任务操作');
    await page.screenshot({ path: testInfo.outputPath('redesign-task-sheet.png') });
    await page.keyboard.press('Escape');
    await page.getByRole('button', { name: '传输选项' }).click();
    await expect(page.getByRole('dialog', { name: '传输选项' })).toBeVisible();
    await expectSheetInViewport('传输选项');
    await page.keyboard.press('Escape');
    // Exercise virtualization with both row sizes, including its final window.
    const tasks = page.getByRole('list', { name: '传输任务' });
    for (const viewport of [
      { width: 568, height: 356, row: 64 },
      { width: 390, height: 844, row: 62 },
    ]) {
      await page.setViewportSize({ width: viewport.width, height: viewport.height });
      const scale = viewport.width === 568 ? 0.6 : 1;
      await expect
        .poll(async () => (await tasks.getByRole('listitem').first().boundingBox())!.height)
        .toBeCloseTo(viewport.row * scale, 1);
      await expect
        .poll(async () => {
          const row = (await tasks.getByRole('listitem').first().boundingBox())!.height;
          return Math.abs((await tasks.locator(':scope > div').boundingBox())!.height - 240 * row);
        })
        .toBeLessThanOrEqual(1);
      if (viewport.width === 568) {
        await expect(tasks.getByRole('listitem').first().locator('.task-meta')).toContainText('已完成');
        await expect(tasks.getByRole('listitem').first().locator('.task-meta')).toContainText('1:00');
        await expect(
          tasks.getByRole('listitem').first().getByRole('button', { name: '播放', exact: true }),
        ).toBeVisible();
      }
      await page.screenshot({ path: testInfo.outputPath(`redesign-transfer-density-${viewport.width}.png`) });
      await tasks.evaluate((list) => {
        list.scrollTop = list.scrollHeight;
      });
      // Fractional rem dimensions can round the scroll extent by less than one pixel.
      await expect(tasks.getByRole('listitem').last()).toBeInViewport({ ratio: 0.99 });
      expect(await tasks.getByRole('listitem').count()).toBeLessThan(40);
      const bounds = (await tasks.boundingBox())!;
      const bottomPadding = await tasks.evaluate((list) => parseFloat(getComputedStyle(list).paddingBottom));
      await expect
        .poll(async () =>
          Math.abs(
            (await tasks.getByRole('listitem').last().boundingBox())!.y +
              viewport.row * scale -
              bounds.y -
              bounds.height +
              bottomPadding,
          ),
        )
        .toBeLessThanOrEqual(1);
      expect(await tasks.evaluate((list) => list.scrollWidth - list.clientWidth)).toBeLessThanOrEqual(0);
    }
    await nav.getByRole('button', { name: '我的' }).click();
    await expect(page.locator('.profile-card')).toBeVisible();
    await expect(page.locator('.workspace-enter-active, .workspace-leave-active')).toHaveCount(0);
    await page.screenshot({ path: testInfo.outputPath('reference-personal.png') });
    await page.getByRole('button', { name: /下载设置/ }).click();
    await expect(page.locator('.settings-category')).toHaveCount(5);
    await expect(page.locator('.mobile-header')).toBeVisible();
    await page.screenshot({ path: testInfo.outputPath('redesign-settings.png') });
    await page.getByRole('button', { name: '下载 目录、速度与任务恢复' }).click();
    await expect(page.getByLabel('全局下载限速（MB/s）', { exact: true })).toBeVisible();
    await page.getByRole('button', { name: '返回设置' }).click();
    await expect(page.locator('.settings-category')).toHaveCount(5);
    await page.getByRole('button', { name: '返回我的' }).click();
    await page
      .getByRole('button', { name: /外观：/ })
      .first()
      .click();
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
});

test('mobile batch selection and removal keep the download action reachable', async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 844 });
  await page.addInitScript(() =>
    Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }),
  );
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
        const icons = Array.from(document.querySelectorAll('.nav-item svg')).filter(
          (svg) => svg.childElementCount > 0,
        ).length;
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
    () =>
      (window as Window & { __BDL_ICON_SAMPLES__?: Array<{ navItems: number; icons: number }> }).__BDL_ICON_SAMPLES__ ??
      [],
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
        const icons = Array.from(document.querySelectorAll('.ui-icon-button svg')).filter(
          (svg) => svg.childElementCount > 0,
        ).length;
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

for (const width of [375, 900, 1280]) {
  test(`appearance hover labels follow theme selection at ${width}px`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width, height: 844 });
    if (width === 375) {
      await page.addInitScript(() => {
        Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' });
      });
    }
    await installTauriMock(page, 'dark');
    await page.goto('/');
    if (width === 375) {
      await page.getByRole('navigation', { name: '主导航' }).getByRole('button', { name: '我的' }).click();
    }
    const appearance = page.getByRole('button', { name: /^外观：/ });
    await expect(appearance).toHaveAttribute('title', '外观：深色');
    for (const label of ['跟随系统', '浅色', '深色']) {
      await appearance.focus();
      await appearance.press('Enter');
      const item = page.getByRole('menuitemcheckbox', { name: label, exact: true });
      await expect(item).toBeVisible();
      await item.focus();
      await item.press('Enter');
      await page.keyboard.press('Escape');
      await expect(page.getByRole('menu')).not.toBeVisible();
      await expect(appearance).toBeFocused();
      await appearance.hover();
      await expect(appearance).toHaveAttribute('title', `外观：${label}`);
      await expect(appearance).toHaveAccessibleName(`外观：${label}`);
      await expect(page.locator('html')).toHaveAttribute('data-theme', label === '深色' ? 'dark' : 'light');
      await page.screenshot({ path: testInfo.outputPath(`appearance-${width}-${label}.png`) });
    }
  });
}

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

  await expect(page.locator('.source-media-row')).toBeVisible();
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
    const runtime = (
      window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> } }
    ).__TAURI_INTERNALS__;
    const invoke = runtime.invoke;
    runtime.invoke = async (command, args) => {
      if (command === 'parse_create_source') Object.assign(window, { __SINGLE_REQUEST__: args });
      return invoke(command, args);
    };
  });
  await input.press('Enter');
  await expect(page.locator('.source-media-row')).toBeVisible();
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
  await expect(page.locator('.source-media-row')).toBeVisible();

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
  await expect(page.getByRole('combobox', { name: '重名处理', exact: true })).toContainText('已有文件则跳过');
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
  await page.getByRole('button', { name: '选择 测试视频' }).click();
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

test('parse result selection uses the list checkbox and compact footer', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await page.getByLabel('Bilibili 链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();

  const results = page.getByRole('list', { name: '解析结果' });
  await expect(results.getByRole('listitem')).toHaveCount(1);
  await expect(results.getByRole('button', { name: '播放 测试视频' })).toBeVisible();
  await results.getByRole('button', { name: '播放 测试视频' }).click();
  await expect(results.getByRole('checkbox', { name: '选择 测试视频' })).not.toBeChecked();
  expect(
    await page.evaluate(
      () =>
        (window as unknown as { __BDL_TEST_INVOKES__: string[] }).__BDL_TEST_INVOKES__.filter(
          (command) => command === 'open_external_url',
        ).length,
    ),
  ).toBe(1);
  await page.getByRole('checkbox', { name: '全选已加载' }).click();
  await expect(page.getByRole('checkbox', { name: '全选已加载' })).toBeChecked();
  const actionBar = page.locator('.selection-action-bar');
  await expect(actionBar.getByRole('button', { name: '取消选择', exact: true })).toBeVisible();
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
    const runtime = (
      window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> } }
    ).__TAURI_INTERNALS__;
    const invoke = runtime.invoke;
    let cancel: (() => void) | undefined;
    runtime.invoke = async (command, args) => {
      if (command === 'parse_load_more')
        return new Promise((_resolve, reject) => {
          cancel = () => reject(new Error('解析已停止'));
        });
      if (command === 'parse_progress') return { active: true, waiting_seconds: 3, queued: false };
      if (command === 'parse_cancel') {
        cancel?.();
        return;
      }
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
  const commands = await page.evaluate(
    () => (window as unknown as { __BDL_TEST_INVOKES__: string[] }).__BDL_TEST_INVOKES__,
  );
  expect(commands.indexOf('selection_create_tasks')).toBeLessThan(commands.indexOf('parse_load_more'));
  await page.screenshot({ path: testInfo.outputPath('background-download.png') });
});

test('batch result selection uses the shared list checkbox', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.goto('/');
  await page.getByLabel('Bilibili 链接或 BV / AV').fill('BV1xx411c7mD\nBV1xx411c7mE');
  await page.getByRole('button', { name: '开始解析' }).click();

  const actionBar = page.locator('.selection-action-bar');
  await expect(actionBar.getByRole('button', { name: '取消选择' })).toBeVisible();
  await expect(actionBar.getByRole('button', { name: /下载/ })).toHaveCount(0);
  await expect(page.locator('.source-function-toolbar').getByRole('button', { name: '下载所选 (2)' })).toBeVisible();
  await expect(page.locator('.batch-result-header').getByRole('button', { name: /全选/ })).toHaveCount(0);
});

test('library uses its title link and makes the whole folder card the enter action', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
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
  await expect(enterFolder.locator('.library-description')).toHaveCount(0);
  await expect(enterFolder.locator('.library-card-meta')).toContainText('个视频');
  const cardHeights = await enterFolder.evaluate((card) => ({
    cover: card.querySelector('.library-cover')?.getBoundingClientRect().height,
    copy: card.querySelector('.library-card-copy')?.getBoundingClientRect().height,
  }));
  expect(cardHeights.cover).toBe(cardHeights.copy);
  await expect(page).toHaveScreenshot('library-index-light-standard.png', {
    animations: 'disabled',
    caret: 'hide',
    maxDiffPixelRatio: 0.01,
  });

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
  await page.getByRole('button', { name: '选择 测试视频' }).click();
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
  await expect(page.getByRole('checkbox', { name: '全选本页' })).toBeVisible();
  await expect(actionBar.getByRole('button', { name: /下载|解析/ })).toHaveCount(0);
  await expect(page.locator('.library-folder-header').getByRole('button', { name: '下载全部' })).toBeVisible();
});

test('subscription collection cards fall back to the collection owner when archive authors are missing', async ({
  page,
}) => {
  await installTauriMock(page, 'light', true, true);
  await page.goto('/');
  await page.getByRole('button', { name: '内容库 收藏与订阅' }).click();
  await page.getByRole('tab', { name: /订阅合集/ }).click();
  await page.getByRole('button', { name: '进入 双赢之路' }).click();

  const firstCard = page.locator('.source-media-row').first();
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
  await expect(page.getByRole('list', { name: '解析结果' })).toHaveCount(0);
});

test('library detail keeps pagination, page count, parsing, and selection on one compact footer row', async ({
  page,
}) => {
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
    const runtime = (
      window as unknown as {
        __TAURI_INTERNALS__: {
          invoke: (command: string, args?: { request?: { page_number?: number } }) => Promise<unknown>;
        };
      }
    ).__TAURI_INTERNALS__;
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
  await page.getByRole('checkbox', { name: '全选本页', exact: true }).check();
  await pagination.getByRole('textbox').fill('8');
  await pagination.getByRole('textbox').press('Enter');
  await expect(page.getByText('第 8 页视频', { exact: true })).toBeVisible();
  await expect(page.locator('.library-selection-count')).toContainText('已选 5');
  await pagination.getByRole('textbox').fill('1');
  await pagination.getByRole('button', { name: '跳转', exact: true }).click();
  await expect(page.getByText('合集视频 1', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => (window as unknown as { __PAGE_REQUESTS__: number[] }).__PAGE_REQUESTS__)).toEqual([
    8,
  ]);
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

test('folder title and video cover expose Bilibili playback', async ({ page }) => {
  await installTauriMock(page, 'light', true, true);
  await page.goto('/');
  await page.getByRole('button', { name: '内容库 收藏与订阅' }).click();
  await page.getByRole('tab', { name: /订阅合集/ }).click();
  await page.getByRole('button', { name: '进入 双赢之路' }).click();

  const folderTitleLink = page.getByRole('button', { name: '在 Bilibili 打开 双赢之路' });
  const videoCoverPlay = page.getByRole('button', { name: '播放 合集视频 1' });
  await expect(folderTitleLink.locator('svg')).toHaveCount(0);
  await expect(videoCoverPlay.locator('svg')).toHaveCount(1);
  await expect(page.getByRole('button', { name: /打开测试用户|打开 测试用户/ })).toHaveCount(0);
});

test('library video rows use checkbox selection and keep the bottom action bar aligned', async ({ page }) => {
  await installTauriMock(page, 'light', true, true);
  await page.goto('/');
  await page.getByRole('button', { name: '内容库 收藏与订阅' }).click();
  await page.getByRole('tab', { name: /订阅合集/ }).click();
  await page.getByRole('button', { name: '进入 双赢之路' }).click();

  const firstCard = page.locator('.source-media-row').first();
  await expect(firstCard).toHaveAttribute('data-selected', 'false');
  await expect(firstCard.getByRole('checkbox', { name: '选择 合集视频 1' })).not.toBeChecked();

  await firstCard.getByRole('button', { name: '选择 合集视频 1' }).click();
  await expect(firstCard).toHaveAttribute('data-selected', 'true');
  await expect(firstCard.getByRole('checkbox', { name: '选择 合集视频 1' })).toBeChecked();

  await firstCard.getByRole('checkbox', { name: '选择 合集视频 1' }).click();
  await expect(firstCard).toHaveAttribute('data-selected', 'false');
  await firstCard.getByRole('checkbox', { name: '选择 合集视频 1' }).click();
  await expect(firstCard).toHaveAttribute('data-selected', 'true');
  const selectedVisualState = await firstCard.evaluate((row) => ({
    border: getComputedStyle(row).borderTopColor,
    background: getComputedStyle(row).backgroundColor,
  }));
  await firstCard.getByRole('checkbox').click();
  expect(
    await firstCard.evaluate((row) => ({
      border: getComputedStyle(row).borderTopColor,
      background: getComputedStyle(row).backgroundColor,
    })),
  ).not.toEqual(selectedVisualState);
  await page.getByRole('checkbox', { name: '全选本页' }).click();
  await expect(page.getByRole('checkbox', { name: '全选本页' })).toBeChecked();

  const verticalCenters = await page.locator('.selection-action-bar').evaluate((footer) => {
    const center = (selector: string) => {
      const element = footer.querySelector<HTMLElement>(selector);
      if (!element) throw new Error(`Missing ${selector}`);
      const rect = element.getBoundingClientRect();
      return rect.top + rect.height / 2;
    };
    return [center('.library-pagination-status'), center('nav')];
  });
  expect(Math.max(...verticalCenters) - Math.min(...verticalCenters)).toBeLessThanOrEqual(1);
});

test('media preferences reorder combinations while download keeps optimal quality', async ({ page }, testInfo) => {
  await installTauriMock(page, 'light');
  await page.addInitScript(() => {
    const target = window as unknown as {
      __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
      __BDL_MEDIA_REQUEST__?: unknown;
      __BDL_MEDIA_SETTINGS__?: unknown;
    };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      if (command === 'settings_update') {
        target.__BDL_MEDIA_SETTINGS__ = args?.settings;
        return args?.settings;
      }
      if (command === 'selection_create_tasks') target.__BDL_MEDIA_REQUEST__ = args?.request;
      return invoke(command, args);
    };
  });
  await page.goto('/');
  await page.getByRole('button', { name: '设置 偏好与维护' }).click();
  await page.getByRole('button', { name: /^下载预设(?: 内容.*)?$/ }).click();
  await page.getByText('优先顺序', { exact: true }).click();
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
  await page.getByRole('button', { name: '选择 测试视频' }).click();
  await page.getByRole('button', { name: '下载所选 (1)' }).click();
  const dialog = page.getByRole('dialog', { name: '下载设置' });
  await expect(dialog.getByRole('combobox', { name: /^下载预设/ })).toContainText('快速下载');
  await expect(dialog.getByRole('combobox', { name: /视频清晰度/ })).toHaveCount(0);
  await dialog.getByRole('button', { name: '开始下载' }).click();
  const result = await page.evaluate(() => {
    const target = window as unknown as {
      __BDL_MEDIA_REQUEST__: { download_preset_id: string };
      __BDL_MEDIA_SETTINGS__: { download_presets: { id: string; workflow: { media_preferences: unknown } }[] };
    };
    return {
      request: target.__BDL_MEDIA_REQUEST__,
      preferences: (target.__BDL_MEDIA_SETTINGS__ as unknown as { media_preferences: unknown }).media_preferences,
    };
  });
  expect(result.request.download_preset_id).toBe('video');
  expect(result.preferences).toEqual({
    video: [
      { quality: '125', codec: 'hevc' },
      { quality: 'sdr', codec: 'auto' },
    ],
    audio: ['30251'],
    fallback: 'best',
  });
});

test('missing directory allows parsing and explicit SDR task creation', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.addInitScript(() => {
    const target = window as unknown as {
      __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
      __BDL_SDR_REQUEST__?: unknown;
    };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      const result = await invoke(command, args);
      if (command === 'environment_health') {
        return {
          ...(result as object),
          ready: false,
          download_directory: { status: 'missing', path: 'C:\\Downloads\\new-folder', message: '开始下载时自动创建' },
        };
      }
      if (command === 'settings_get') {
        return {
          ...(result as object),
          quality: 'sdr',
          media_preferences: { video: [], audio: ['30280'], fallback: 'best' },
        };
      }
      if (command === 'selection_create_tasks') target.__BDL_SDR_REQUEST__ = args?.request;
      return result;
    };
  });
  await page.goto('/');
  await page.getByLabel('链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();
  await page.getByRole('button', { name: '选择 测试视频' }).click();
  await page.getByRole('button', { name: '下载所选 (1)' }).click();
  const dialog = page.getByRole('dialog', { name: '下载设置' });
  await expect(dialog.getByRole('combobox', { name: /^下载预设/ })).toContainText('快速下载');
  await dialog.getByRole('button', { name: '开始下载' }).click();
  const result = await page.evaluate(() => {
    const target = window as unknown as {
      __BDL_SDR_REQUEST__: { download_preset_id: string; quality?: string };
      __BDL_TEST_INVOKES__: string[];
    };
    return {
      request: target.__BDL_SDR_REQUEST__,
      creates: target.__BDL_TEST_INVOKES__.filter((name) => name === 'environment_create_download_directory').length,
    };
  });
  expect(result.request.download_preset_id).toBe('video');
  expect(result.request.quality).toBeUndefined();
  expect(result.creates).toBe(0);
});

test('download tabs use saved presets and keep overrides local', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1100, height: 900 });
  await installTauriMock(page, 'light');
  await page.addInitScript(() => {
    const target = window as unknown as {
      __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
      __BDL_PRESET_REQUEST__?: unknown;
    };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      const result = await invoke(command, args);
      if (command === 'settings_get')
        return {
          ...(result as object),
          naming_template: '{title}/{part_title}.{ext}',
          naming_presets: [{ id: 'favorites', name: '我的收藏', template: '{title}.{ext}' }],
        };
      if (command === 'selection_create_tasks') target.__BDL_PRESET_REQUEST__ = args?.request;
      return result;
    };
  });
  await page.goto('/');
  await page.getByLabel('链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();
  await page.getByRole('button', { name: '选择 测试视频' }).click();
  await page.getByRole('button', { name: '下载所选 (1)' }).click();
  const dialog = page.getByRole('dialog', { name: '下载设置' });
  await expect(dialog.getByRole('tab', { name: '下载', exact: true })).toHaveAttribute('aria-selected', 'true');
  await dialog.getByRole('combobox', { name: '命名预设', exact: true }).click();
  await page.getByRole('option', { name: '我的收藏', exact: true }).click();
  await dialog.getByRole('combobox', { name: /重名处理/ }).click();
  await page.getByRole('option', { name: '扩展文件名', exact: true }).click();
  await dialog.getByRole('button', { name: '取消', exact: true }).click();
  await page.getByRole('button', { name: '下载所选 (1)' }).click();
  await expect(dialog.getByRole('combobox', { name: /重名处理/ })).toContainText('已有文件则跳过');
  await expect(dialog.getByRole('textbox', { name: '命名模板', exact: true })).toHaveValue(
    '{title}/{part_title}.{ext}',
  );
  await dialog.getByRole('combobox', { name: '命名预设', exact: true }).click();
  await page.getByRole('option', { name: '我的收藏', exact: true }).click();
  await dialog.getByRole('combobox', { name: /重名处理/ }).click();
  await page.getByRole('option', { name: '扩展文件名', exact: true }).click();
  await expect(dialog.getByRole('combobox', { name: '命名预设', exact: true })).toContainText('我的收藏');
  await expect(page.getByRole('listbox')).toHaveCount(0);
  await page.screenshot({ path: testInfo.outputPath('download-general.png') });
  const generalBounds = await dialog.boundingBox();
  await dialog.getByRole('tab', { name: '调度', exact: true }).click();
  await expect.poll(async () => (await dialog.boundingBox())?.height).toBe(generalBounds?.height);
  await expect.poll(async () => (await dialog.boundingBox())?.y).toBe(generalBounds?.y);
  await page.screenshot({ path: testInfo.outputPath('download-media.png') });
  await page.setViewportSize({ width: 900, height: 600 });
  await dialog.getByRole('tab', { name: '下载', exact: true }).click();
  const scrollArea = dialog.locator('.download-options-scroll');
  await expect.poll(() => scrollArea.evaluate((element) => element.scrollHeight > element.clientHeight)).toBe(true);
  await scrollArea.evaluate((element) => {
    element.scrollTop = element.scrollHeight;
  });
  await expect.poll(() => scrollArea.evaluate((element) => element.scrollTop)).toBeGreaterThan(0);
  await expect(dialog.getByRole('button', { name: '恢复默认偏好' })).toBeInViewport();
  await expect(dialog.getByRole('tab', { name: '下载', exact: true })).toBeInViewport();
  await page.screenshot({ path: testInfo.outputPath('download-scroll.png') });
  await dialog.getByRole('button', { name: '开始下载' }).click();
  const result = await page.evaluate(() => {
    const target = window as unknown as {
      __BDL_PRESET_REQUEST__: { naming_template: string; duplicate_naming_strategy: string };
      __BDL_TEST_INVOKES__: string[];
    };
    return {
      request: target.__BDL_PRESET_REQUEST__,
      saves: target.__BDL_TEST_INVOKES__.filter((name) => name === 'settings_update').length,
    };
  });
  expect(result.request.naming_template).toBe('{title}.{ext}');
  expect(result.request.duplicate_naming_strategy).toBe('append_suffix');
  expect(result.saves).toBe(0);
});

test('saved naming presets are reusable while unsaved defaults stay in settings', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1100, height: 900 });
  await installTauriMock(page, 'light');
  await page.addInitScript(() => {
    const target = window as unknown as {
      __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
    };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) =>
      command === 'settings_update' ? args?.settings : invoke(command, args);
  });
  await page.goto('/');
  await page.getByRole('button', { name: '设置 偏好与维护' }).click();
  await page.getByRole('button', { name: /文件命名/ }).click();
  await page.getByRole('textbox', { name: '命名模板', exact: true }).fill('{owner_name}/{title}');
  await expect(page.getByText('文件名预览：')).toContainText('示例UP/示例视频.mp4');
  await page.getByRole('button', { name: '插入魔法变量', exact: true }).click();
  await expect(page.locator('[data-variable="ext"]')).toHaveCount(0);
  await page.keyboard.press('Escape');
  await page.getByRole('textbox', { name: '预设名称', exact: true }).fill('按 UP 收藏');
  await page.getByRole('button', { name: '保存为预设', exact: true }).click();
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByRole('button', { name: '保存', exact: true })).toBeDisabled();
  await page.screenshot({ path: testInfo.outputPath('naming-presets-settings.png') });
  await page.getByRole('textbox', { name: '命名模板', exact: true }).fill('{bvid}.{ext}');
  await page.getByRole('button', { name: '解析 添加与选择' }).click();
  await page.getByRole('dialog', { name: '设置尚未保存' }).getByRole('button', { name: '放弃更改' }).click();
  await page.getByLabel('链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();
  await page.getByRole('button', { name: '选择 测试视频' }).click();
  await page.getByRole('button', { name: '下载所选 (1)' }).click();
  const dialog = page.getByRole('dialog', { name: '下载设置' });
  await expect(dialog.getByRole('combobox', { name: '命名预设', exact: true })).toContainText('按 UP 收藏');
  await expect(dialog.getByText('文件名预览：')).toContainText('示例UP/示例视频.mp4');
});

test('download archive settings inherit defaults and override processing per task', async ({ page }, testInfo) => {
  await installTauriMock(page, 'light');
  await page.addInitScript(() => {
    const target = window as unknown as {
      __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
      __BDL_ARCHIVE_REQUEST__?: unknown;
    };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      const result = await invoke(command, args);
      if (command === 'settings_get')
        return {
          ...(result as object),
          output_extension: 'mkv',
          archive_mode: 'custom',
          retain_raw_streams: true,
          embed_cover: true,
          embed_subtitles: true,
        };
      if (command === 'selection_create_tasks') target.__BDL_ARCHIVE_REQUEST__ = args?.request;
      return result;
    };
  });
  await page.goto('/');
  await page.getByLabel('链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();
  await page.getByRole('button', { name: '选择 测试视频' }).click();
  await page.getByRole('button', { name: '下载所选 (1)' }).click();
  const dialog = page.getByRole('dialog', { name: '下载设置' });
  await expect(dialog.getByRole('combobox', { name: /^下载预设/ })).toContainText('历史配置');
  await expect(dialog.getByLabel('本次输出预览')).toContainText('内含封面、字幕');
  await expect(dialog.getByRole('checkbox', { name: '嵌入字幕', exact: true })).toHaveCount(0);
  await dialog.getByRole('combobox', { name: /^下载预设/ }).click();
  await page.getByRole('option', { name: '快速下载', exact: true }).click();
  await page.screenshot({ path: testInfo.outputPath('download-archive.png') });
  await dialog.getByRole('button', { name: '开始下载' }).click();
  const result = await page.evaluate(() => {
    const target = window as unknown as {
      __BDL_ARCHIVE_REQUEST__: Record<string, unknown>;
      __BDL_TEST_INVOKES__: string[];
    };
    return {
      request: target.__BDL_ARCHIVE_REQUEST__,
      saves: target.__BDL_TEST_INVOKES__.filter((name) => name === 'settings_update').length,
    };
  });
  expect(result.request).toMatchObject({ download_preset_id: 'video' });
  expect(result.request.embed_subtitles).toBeUndefined();
  expect(result.saves).toBe(1);
});

const openPresetSettings = async (page: Page) => {
  const desktop = page.getByRole('button', { name: /^设置(?: 偏好与维护| · Ctrl\+4)$/ });
  if (await desktop.isVisible()) await desktop.click();
  else {
    await page.getByRole('navigation', { name: '主导航' }).getByRole('button', { name: '我的', exact: true }).click();
    await page.getByRole('button', { name: /下载设置/ }).click();
  }
  await page.getByRole('button', { name: /^下载预设(?: 内容.*)?$/ }).click();
};
const installPresetPersistence = async (page: Page) => {
  await page.addInitScript(() => {
    const target = window as unknown as {
      __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
      __FORMAT_REQUEST__?: unknown;
    };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      if (command === 'settings_get') {
        const defaults = await invoke(command, args);
        const saved = localStorage.getItem('fixture-download-settings');
        return saved ? JSON.parse(saved) : defaults;
      }
      if (command === 'settings_update') {
        localStorage.setItem('fixture-download-settings', JSON.stringify(args?.settings));
        return args?.settings;
      }
      if (command === 'selection_create_tasks') target.__FORMAT_REQUEST__ = args?.request;
      return invoke(command, args);
    };
  });
};

for (const theme of ['light', 'dark'] as const) {
  test(`settings cleanup ${theme} desktop media and direct attachments`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await installTauriMock(page, theme);
    await installPresetPersistence(page);
    await page.goto('/');
    await openPresetSettings(page);
    await expect(page.getByRole('checkbox')).toHaveCount(0);
    await expect(page.getByLabel('预设概况')).toContainText('MP4 音频+视频');
    await expect(page.getByRole('combobox', { name: '视频清晰度', exact: true })).toBeVisible();
    await page.getByRole('combobox', { name: '视频清晰度', exact: true }).click();
    await page.getByRole('option', { name: /1080P.*80/, exact: false }).click();
    await page.getByRole('button', { name: '保存', exact: true }).click();
    await page.getByRole('combobox', { name: '下载预设', exact: true }).click();
    await page.getByRole('option', { name: '下载全部资源', exact: true }).click();
    await expect(page.getByRole('combobox', { name: '视频清晰度', exact: true })).toContainText('1080P');
    await page.getByRole('combobox', { name: '下载预设', exact: true }).click();
    await page.getByRole('option', { name: '快速下载', exact: true }).click();
    await page.screenshot({ path: testInfo.outputPath('presets-overview.png'), animations: 'disabled' });
    await page.getByRole('button', { name: '编辑', exact: true }).click();
    const editor = page.getByRole('dialog', { name: '编辑预设', exact: true });
    await expect(editor.getByRole('tab')).toHaveCount(3);
    await expect(editor.getByRole('combobox', { name: /视频清晰度/ })).toHaveCount(0);
    const cover = editor.getByRole('checkbox', { name: '下载封面', exact: true });
    await cover.focus();
    await page.keyboard.press('Space');
    await editor.getByRole('checkbox', { name: '下载字幕', exact: true }).check();
    await editor.getByRole('tab', { name: '封装', exact: true }).click();
    const embed = editor.getByRole('checkbox', { name: '嵌入字幕', exact: true });
    await embed.focus();
    await page.keyboard.press('Space');
    const format = editor.getByRole('combobox', { name: '封装格式', exact: true });
    await expect(format).toContainText('MKV');
    await editor.getByRole('checkbox', { name: '嵌入封面', exact: true }).check();
    await format.click();
    await page.getByRole('option', { name: 'MP4', exact: true }).click();
    await expect(embed).not.toBeChecked();
    await expect(editor.getByRole('checkbox', { name: '嵌入封面', exact: true })).not.toBeChecked();
    await embed.check();
    await page.setViewportSize({ width: 900, height: 700 });
    await expect(editor.getByRole('button', { name: '保存预设', exact: true })).toBeInViewport();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: testInfo.outputPath('preset-editor.png'), animations: 'disabled' });
    await editor.getByRole('button', { name: '保存预设', exact: true }).click();
    await expect(editor).not.toBeVisible();
    await expect(page.getByLabel('预设概况')).toContainText('MKV 音频+视频（内含字幕）');
    const saved = await page.evaluate(() => JSON.parse(localStorage.getItem('fixture-download-settings') || '{}'));
    expect(saved.download_presets.find((p: { id: string }) => p.id === 'video').workflow).toMatchObject({
      container: 'mkv',
      cover: { enabled: true, save: true, embed: false },
      subtitles: { enabled: true, save: true, embed: true },
    });
  });

  test(`settings cleanup ${theme} one-time download options`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await installTauriMock(page, theme);
    await page.addInitScript(() => {
      const target = window as unknown as {
        __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
        __CLEAN_REQUEST__?: unknown;
      };
      const invoke = target.__TAURI_INTERNALS__.invoke;
      target.__TAURI_INTERNALS__.invoke = async (command, args) => {
        if (command === 'selection_create_tasks') target.__CLEAN_REQUEST__ = args?.request;
        return invoke(command, args);
      };
    });
    await page.goto('/');
    await page.getByLabel('链接或 BV / AV').fill('BV1xx411c7mD');
    await page.getByRole('button', { name: '开始解析' }).click();
    await page.getByRole('button', { name: '选择 测试视频' }).click();
    await page.getByRole('button', { name: '下载所选 (1)' }).click();
    const dialog = page.getByRole('dialog', { name: '下载设置' });
    await expect(dialog.getByRole('tab')).toHaveCount(2);
    for (const name of ['下载内容', '封装格式', '视频清晰度', '字幕格式', '弹幕格式'])
      await expect(dialog.getByRole('combobox', { name: new RegExp('^' + name) })).toHaveCount(0);
    await expect(dialog.getByRole('checkbox')).toHaveCount(0);
    await dialog.getByRole('combobox', { name: /^下载预设/ }).click();
    await page.getByRole('option', { name: '封装 MKV（视频+字幕）', exact: true }).click();
    await expect(dialog.getByLabel('本次输出预览')).toContainText('内含字幕');
    await page.setViewportSize({ width: 900, height: 700 });
    await expect(dialog.getByRole('button', { name: '开始下载', exact: true })).toBeInViewport();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: testInfo.outputPath('download-presets.png'), animations: 'disabled' });
    await dialog.getByRole('button', { name: '开始下载', exact: true }).click();
    await expect
      .poll(() => page.evaluate(() => (window as unknown as { __CLEAN_REQUEST__?: unknown }).__CLEAN_REQUEST__))
      .toMatchObject({ download_preset_id: 'mkv' });
  });
}

for (const theme of ['light', 'dark'] as const) {
  test(`settings cleanup ${theme} mobile attachments and MKV`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width: 375, height: 844 });
    await page.addInitScript(() =>
      Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }),
    );
    await installTauriMock(page, theme);
    await installPresetPersistence(page);
    await page.goto('/');
    await openPresetSettings(page);
    await expect(page.getByRole('checkbox')).toHaveCount(0);
    await page.screenshot({ path: testInfo.outputPath('mobile-preset-overview.png'), animations: 'disabled' });
    await page.getByRole('combobox', { name: '下载预设', exact: true }).click();
    await page.getByRole('option', { name: '封装 MKV（视频+字幕）', exact: true }).click();
    await page.getByRole('button', { name: '编辑', exact: true }).click();
    const editor = page.getByRole('dialog', { name: '编辑预设', exact: true });
    await editor.getByRole('tab', { name: '封装', exact: true }).click();
    const embed = editor.getByRole('checkbox', { name: '嵌入字幕', exact: true });
    await expect(embed).toBeChecked();
    await embed.uncheck();
    await expect(embed).toBeDisabled();
    await editor.getByRole('tab', { name: '转换', exact: true }).click();
    await editor.getByRole('checkbox', { name: '保存独立字幕', exact: true }).check();
    await expect(editor.getByRole('button', { name: '保存预设', exact: true })).toBeInViewport();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: testInfo.outputPath('mobile-preset-editor.png'), animations: 'disabled' });
    await editor.getByRole('button', { name: '保存预设', exact: true }).click();
    await expect(editor).not.toBeVisible();
    await page.getByRole('button', { name: '返回设置', exact: true }).click();
    await page.getByRole('button', { name: /下载 目录/ }).click();
    await expect(page.getByRole('combobox', { name: '单任务分段数', exact: true })).toBeVisible();
    await expect(page.getByRole('textbox', { name: 'FFmpeg 路径', exact: true })).toHaveCount(0);
  });
}

test('pagination presets save rules and allow custom cooldown', async ({ page }, testInfo) => {
  await installTauriMock(page, 'light');
  await page.addInitScript(() => {
    const target = window as unknown as {
      __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
      __RULES__?: unknown;
    };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      if (command === 'settings_update') {
        target.__RULES__ = (args?.settings as { parse_rules: unknown }).parse_rules;
        return args?.settings;
      }
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
  expect(await page.evaluate(() => (window as unknown as { __RULES__: unknown }).__RULES__)).toEqual({
    pages_per_round: 5,
    interval_seconds: 1,
    rest_seconds: 10,
  });
  await page.screenshot({ path: testInfo.outputPath('pagination-presets.png') });
});

for (const scenario of ['confirmed', 'first-run', 'save-failure'] as const) {
  test(`native preferences startup without browser preference storage: ${scenario}`, async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (error) => errors.push(error.message));
    await installTauriMock(page, 'dark');
    await page.addInitScript(
      ({ scenario }) => {
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
            return { ...((await invoke(command)) as object), usage_notice_acknowledged: scenario === 'confirmed' };
          }
          if (command === 'settings_update') {
            if (scenario === 'save-failure') throw new Error('Settings are read only');
            target.__SAVED_PREFERENCES__ = args?.settings;
            return args?.settings;
          }
          return invoke(command, args);
        };
      },
      { scenario },
    );
    await page.goto('/');
    await expect(page.locator('.nav-item')).toHaveCount(5);
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
    if (scenario !== 'confirmed') {
      await page.getByRole('button', { name: '我已阅读并了解，继续使用' }).click();
      await expect(page.getByRole('dialog', { name: '使用前请阅读' })).toBeHidden();
      if (scenario === 'first-run') {
        await expect
          .poll(() =>
            page.evaluate(
              () =>
                (window as unknown as { __SAVED_PREFERENCES__?: { usage_notice_acknowledged: boolean } })
                  .__SAVED_PREFERENCES__?.usage_notice_acknowledged,
            ),
          )
          .toBe(true);
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

for (const theme of ['light', 'dark'] as const) {
  test(`unified desktop media lists preserve metadata and contextual playback in ${theme}`, async ({
    page,
  }, testInfo) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await installTauriMock(page, theme, true, true);
    const longTitle =
      '一个足够长的视频标题：从城市的日常风景到遥远星空，记录每一个值得珍藏的瞬间与故事 · 完整专题纪录片';
    await page.addInitScript(
      ({ title }) => {
        const target = window as unknown as {
          __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> };
        };
        const original = target.__TAURI_INTERNALS__.invoke;
        target.__TAURI_INTERNALS__.invoke = async (command, args) => {
          const result = await original(command, args);
          if (command === 'queue_list')
            return Array.from({ length: 6 }, (_, index): DownloadTask => ({
              id: `design-task-${index}`,
              title: index === 0 ? title : `城市与远方 · 第 ${index + 1} 集`,
              source_id: 'video:fixture',
              status: index === 1 ? 'downloading' : 'completed',
              resources: [],
              output_path: `C:/Downloads/城市与远方/${index + 1}.mp4`,
              refresh_intent: {
                input: { kind: 'video_bvid', bvid: index === 0 ? 'BV1xx411c7mD' : `BVfixture${index}` },
                cid: index === 0 ? 2 : index + 10,
                cover_url: 'https://i0.hdslb.com/design-cover.svg',
                duration_seconds: 1132 + index * 80,
              },
              media_selection: { video_quality: '80', audio_quality: 'best', video_codec: 'avc', container: 'mp4' },
              scheduled_at: null,
              speed_limit_bytes_per_second: null,
            }));
          if (command === 'queue_logs') return [];
          if (command === 'queue_output_sizes')
            return Object.fromEntries((args as { taskIds: string[] }).taskIds.map((id) => [id, 128 * 1024 * 1024]));
          if (command === 'parse_create_source') {
            const tree = result as NormalizedSourceTree;
            tree.groups.forEach((group) =>
              group.items.forEach((item, index) => {
                item.title = index === 0 ? title : `城市与远方 · 第 ${index + 1} 集`;
                item.owner_name = '城市观察工作室';
                item.publish_date = '2026-09-27';
                item.cover_url = 'https://i0.hdslb.com/design-cover.svg';
                item.duration_seconds = 1132;
                item.parts.forEach((part) => {
                  part.title = item.title;
                  part.duration_seconds = 1132;
                });
              }),
            );
            return tree;
          }
          return result;
        };
      },
      { title: longTitle },
    );
    await page.route('https://i0.hdslb.com/design-cover.svg', (route) =>
      route.fulfill({
        contentType: 'image/svg+xml',
        body: '<svg xmlns="http://www.w3.org/2000/svg" width="480" height="270"><rect width="480" height="270" fill="#9bb7bb"/><path d="M0 200L110 90L220 170L345 45L480 170V270H0Z" fill="#466a75"/><path d="M0 225L180 155L360 230L480 140V270H0Z" fill="#233f4b"/><circle cx="390" cy="55" r="24" fill="#f2d9b2"/></svg>',
      }),
    );
    await page.goto('/');
    await page.getByLabel('Bilibili 链接或 BV / AV').fill('BV1xx411c7mD');
    await page.getByRole('button', { name: '开始解析' }).click();
    const parsed = page.locator('.source-media-row').first();
    await expect(parsed.locator('.media-author')).toContainText('城市观察工作室');
    await expect(parsed.locator('.media-date')).toHaveText('2026-09-27');
    await expect(parsed.locator('.media-size')).toContainText('128 MB');
    await expect(parsed.locator('.media-cover img')).toBeVisible();
    expect(await parsed.locator('.media-title').evaluate((el) => getComputedStyle(el).whiteSpace)).toBe('nowrap');
    await expect(page.getByText('选择内容', { exact: true })).toHaveCount(0);
    await parsed.getByRole('button', { name: `播放本地文件 ${longTitle}` }).click();
    await expect(parsed.getByRole('checkbox')).not.toBeChecked();
    expect(
      await page.evaluate(
        () =>
          (window as unknown as { __BDL_TEST_INVOKES__: string[] }).__BDL_TEST_INVOKES__.filter(
            (command) => command === 'queue_open_file',
          ).length,
      ),
    ).toBe(1);
    await parsed.locator('.media-title').hover();
    const tooltip = page.locator('[data-slot="text"]').filter({ hasText: longTitle });
    await expect(tooltip).toBeVisible();
    await expect(tooltip).toHaveCSS('white-space', 'normal');
    expect(await tooltip.evaluate((el) => el.getBoundingClientRect().height)).toBeGreaterThan(20);
    await page.mouse.move(0, 0);
    await page.screenshot({ path: testInfo.outputPath(`unified-parse-${theme}.png`) });
    await page.getByRole('button', { name: '内容库 收藏与订阅' }).click();
    await page.getByRole('tab', { name: /订阅合集/ }).click();
    await page.getByRole('button', { name: '进入 双赢之路' }).click();
    await expect(page.locator('.library-panel').getByRole('tablist')).toHaveCount(0);
    const libraryRow = page.locator('.source-media-row').first();
    await expect(libraryRow.locator('.media-title')).toHaveText(longTitle);
    await libraryRow.getByRole('button', { name: `播放 ${longTitle}`, exact: true }).click();
    await expect(libraryRow.getByRole('checkbox')).not.toBeChecked();
    await page.screenshot({ path: testInfo.outputPath(`unified-library-${theme}.png`) });
    await page.keyboard.press('Control+3');
    await page.getByRole('tab', { name: /全部/ }).click();
    await expect(page.locator('.task-table-row')).toHaveCount(6);
    await expect(page.locator('.task-table-row').first().locator('.media-size')).toContainText('128 MB');
    await page.locator('.task-table-row').first().getByRole('button', { name: '更多操作' }).click();
    await expect(page.getByRole('menuitem', { name: '打开文件夹' })).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(page.getByRole('menuitem', { name: '打开文件夹' })).toBeHidden();
    await page.screenshot({ path: testInfo.outputPath(`unified-transfer-${theme}.png`) });
    await page.setViewportSize({ width: 900, height: 700 });
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(900);
    const overflow = await page
      .locator('.task-table-row')
      .first()
      .evaluate((row) => row.scrollWidth - row.clientWidth);
    expect(overflow).toBeLessThanOrEqual(1);
    await page.screenshot({ path: testInfo.outputPath(`unified-transfer-${theme}-narrow.png`) });
  });
}

test('ten thousand parsed rows stay virtual while selection covers the whole source', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await installTauriMock(page, 'light');
  await page.addInitScript(() => {
    const target = window as unknown as {
      __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> };
    };
    const original = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      const result = await original(command, args);
      if (command !== 'parse_create_source') return result;
      const tree = result as NormalizedSourceTree;
      const template = tree.groups[0].items[0];
      tree.source.kind = 'collection';
      tree.source.loaded_count = 10000;
      tree.source.total_count = 10000;
      tree.source.has_more = false;
      tree.groups[0].items = Array.from({ length: 10000 }, (_, index) => ({
        ...template,
        id: `item:large:${index}`,
        title: `万条视频 ${index + 1}`,
        parts: [{ ...template.parts[0], id: `part:large:${index}`, title: `万条视频 ${index + 1}` }],
      }));
      return tree;
    };
  });
  await page.goto('/');
  await page.getByLabel('Bilibili 链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();
  const rows = page.locator('.source-media-row');
  await expect(rows.first()).toHaveAttribute('aria-setsize', '10000');
  expect(await rows.count()).toBeLessThan(30);
  await page.getByRole('checkbox', { name: '全选已加载' }).check();
  await expect(page.getByRole('button', { name: '下载所选 (10000)' })).toBeEnabled();
  const body = page.locator('.source-list-body');
  await body.evaluate((el) => {
    el.scrollTop = el.scrollHeight;
    el.dispatchEvent(new Event('scroll'));
  });
  await expect(rows.last()).toHaveAttribute('aria-posinset', '10000');
  await expect(rows.last().getByRole('checkbox')).toBeChecked();
  await expect(rows.last()).toBeInViewport();
  expect(await rows.count()).toBeLessThan(30);
  await rows.last().getByRole('checkbox').uncheck();
  await expect(page.getByRole('button', { name: '下载所选 (9999)' })).toBeEnabled();
  await body.evaluate((el) => {
    el.scrollTop = 0;
    el.dispatchEvent(new Event('scroll'));
  });
  await expect(rows.first()).toHaveAttribute('aria-posinset', '1');
  await expect(rows.first().getByRole('checkbox')).toBeChecked();
  await page.setViewportSize({ width: 1280, height: 1800 });
  const rendered = await rows.count();
  expect(rendered).toBeLessThan(35);
  await expect
    .poll(() =>
      body.evaluate((el) => {
        const bounds = el.getBoundingClientRect();
        const children = el.querySelectorAll('.source-media-row');
        return children[children.length - 1].getBoundingClientRect().bottom >= bounds.bottom;
      }),
    )
    .toBe(true);
});

for (const [format, width, theme, danmakuFormat] of [
  ['m4s', 1280, 'light', 'HTML（离线播放）'],
  ['mp3', 900, 'dark', 'SRT（字幕显示）'],
  ['mp3', 375, 'light', 'ASS（滚动弹幕）'],
] as const) {
  test(`download recipe remembers ${format} at ${width}px`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width, height: 900 });
    await installTauriMock(page, theme);
    await installPresetPersistence(page);
    await page.goto('/');
    await openPresetSettings(page);
    await page.getByRole('button', { name: '新增', exact: true }).click();
    const editor = page.getByRole('dialog', { name: '新增预设', exact: true });
    await editor.getByRole('textbox', { name: '预设名称', exact: true }).fill('我的音频与字幕');
    await editor.getByRole('checkbox', { name: '下载视频', exact: true }).uncheck();
    await editor.getByRole('checkbox', { name: '下载字幕', exact: true }).check();
    await editor.getByRole('checkbox', { name: '下载弹幕', exact: true }).check();
    await editor.getByRole('tab', { name: '转换', exact: true }).click();
    await editor.getByRole('combobox', { name: '独立音频格式', exact: true }).click();
    await page.getByRole('option', { name: format === 'm4s' ? '不转换（m4s）' : '转为 MP3', exact: true }).click();
    await editor.getByRole('combobox', { name: '字幕格式', exact: true }).click();
    await page.getByRole('option', { name: 'ASS', exact: true }).click();
    await editor.getByRole('combobox', { name: '弹幕格式', exact: true }).click();
    await page.getByRole('option', { name: danmakuFormat, exact: true }).click();
    await editor.getByRole('checkbox', { name: '额外保留原始弹幕（XML）', exact: true }).check();
    await page.screenshot({ path: testInfo.outputPath('recipe-editor.png'), animations: 'disabled' });
    await editor.getByRole('button', { name: '保存预设', exact: true }).click();
    await expect(editor).not.toBeVisible();
    await expect(page.getByLabel('预设概况')).toContainText('字幕 ASS');
    const desktopParse = page.getByRole('button', { name: /^解析(?: 添加与选择| · Ctrl\+1)$/ });
    if (await desktopParse.isVisible()) await desktopParse.click();
    else
      await page.getByRole('navigation', { name: '主导航' }).getByRole('button', { name: '解析', exact: true }).click();
    const openDownload = async () => {
      await page.getByLabel('链接或 BV / AV').fill('BV1xx411c7mD');
      await page.getByRole('button', { name: '开始解析' }).click();
      await page.getByRole('button', { name: '选择 测试视频' }).click();
      await page.getByRole('button', { name: '下载所选 (1)' }).click();
    };
    await openDownload();
    const dialog = page.getByRole('dialog', { name: '下载设置' });
    const preset = dialog.getByRole('combobox', { name: /^下载预设/ });
    await expect(preset).toContainText('我的音频与字幕');
    await expect(dialog.getByLabel('本次输出预览')).toContainText(`独立音频 ${format.toUpperCase()}`);
    await expect(dialog.getByLabel('本次输出预览')).toContainText(`弹幕 ${danmakuFormat.split('（')[0]}`);
    await expect(dialog.getByLabel('本次输出预览')).toContainText('原始弹幕');
    await dialog.getByRole('button', { name: '取消', exact: true }).click();
    await page.getByRole('button', { name: '下载所选 (1)' }).click();
    await expect(preset).toContainText('我的音频与字幕');
    await page.screenshot({ path: testInfo.outputPath('recipe-download.png'), animations: 'disabled' });
    await dialog.getByRole('button', { name: '开始下载', exact: true }).click();
    const saved = await page.evaluate(() => JSON.parse(localStorage.getItem('fixture-download-settings') || '{}'));
    await expect
      .poll(() => page.evaluate(() => (window as unknown as { __FORMAT_REQUEST__: unknown }).__FORMAT_REQUEST__))
      .toMatchObject({ download_preset_id: saved.selected_download_preset });
    await page.reload();
    await openDownload();
    await expect(preset).toContainText('我的音频与字幕');
    await preset.click();
    await page.getByRole('option', { name: '快速下载', exact: true }).click();
    await dialog.getByRole('button', { name: '取消', exact: true }).click();
    await page.getByRole('button', { name: '下载所选 (1)' }).click();
    await expect(preset).toContainText('快速下载');
  });
}

test('custom preset creation validates names and deleting the default selects a saved fallback', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await installTauriMock(page, 'light');
  await installPresetPersistence(page);
  await page.goto('/');
  await openPresetSettings(page);
  await page.getByRole('button', { name: '新增', exact: true }).click();
  const editor = page.getByRole('dialog', { name: '新增预设', exact: true });
  const name = editor.getByRole('textbox', { name: '预设名称', exact: true });
  const save = editor.getByRole('button', { name: '保存预设', exact: true });
  await name.fill('快速下载');
  await expect(save).toBeDisabled();
  await name.fill('');
  await expect(save).toBeDisabled();
  await name.fill('学习字幕');
  await editor.getByRole('checkbox', { name: '下载视频', exact: true }).uncheck();
  await editor.getByRole('checkbox', { name: '下载音频', exact: true }).uncheck();
  await expect(save).toBeDisabled();
  await editor.getByRole('checkbox', { name: '下载字幕', exact: true }).check();
  await save.click();
  await expect(editor).not.toBeVisible();
  await expect(page.getByLabel('预设概况')).toContainText('字幕 SRT');
  await page.getByRole('button', { name: '编辑', exact: true }).click();
  const editing = page.getByRole('dialog', { name: '编辑预设', exact: true });
  await editing.getByRole('textbox', { name: '预设名称', exact: true }).fill('取消不改');
  await page.keyboard.press('Escape');
  await expect(editing).not.toBeVisible();
  await expect(page.getByRole('combobox', { name: '下载预设', exact: true })).toContainText('学习字幕');
  await page.getByRole('button', { name: '编辑', exact: true }).click();
  await editing.getByRole('button', { name: '删除预设', exact: true }).click();
  await expect(editing).not.toBeVisible();
  const after = await page.evaluate(() => JSON.parse(localStorage.getItem('fixture-download-settings') || '{}'));
  expect(after.selected_download_preset).toBe('video');
  expect(after.download_presets.some((p: { name: string }) => p.name === '学习字幕')).toBe(false);
});

test('offline HTML danmaku renders safe text and synchronizes seeking', async ({ page }) => {
  const template = await readFile(
    new URL('../../../../crates/bdl-core/src/danmaku-viewer.html', import.meta.url),
    'utf8',
  );
  const xml = '<i><d p="1,1,25,16777215">中文弹幕</d><d p="2,5,25,0">&lt;img src=x onerror=alert(1)&gt;</d></i>';
  await page.setContent(template.replace('__BDL_XML__', JSON.stringify(xml).replaceAll('<', '\\u003c')));
  await expect(page.locator('#status')).toHaveText('共 2 条弹幕');
  await expect(page.locator('#rows')).toContainText('<img src=x onerror=alert(1)>');
  await expect(page.locator('img')).toHaveCount(0);
  await page.locator('#seek').evaluate((input: HTMLInputElement) => {
    input.value = '2';
    input.dispatchEvent(new Event('input'));
  });
  await expect(page.locator('#overlay')).toContainText('中文弹幕');
  await expect(page.locator('#overlay')).toContainText('<img src=x onerror=alert(1)>');
  await expect(page.locator('#overlay .comment').last()).toHaveCSS('color', 'rgb(0, 0, 0)');
});

for (const theme of ['light', 'dark'] as const) {
  test(`missing FFmpeg download link ${theme}`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width: 900, height: 700 });
    await installTauriMock(page, theme, false);
    await page.addInitScript(() => {
      const target = window as unknown as {
        __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
        __FFMPEG_DOWNLOAD_URL__?: unknown;
      };
      const invoke = target.__TAURI_INTERNALS__.invoke;
      target.__TAURI_INTERNALS__.invoke = async (command, args) => {
        if (command === 'open_external_url') target.__FFMPEG_DOWNLOAD_URL__ = args?.url;
        return invoke(command, args);
      };
    });
    await page.goto('/');
    await page.keyboard.press('Control+4');
    const download = page.getByRole('button', { name: '下载 FFmpeg', exact: true });
    await download.scrollIntoViewIfNeeded();
    await expect(download).toBeVisible();
    await page.screenshot({ path: testInfo.outputPath('ffmpeg-download.png'), animations: 'disabled' });
    await download.click();
    await expect.poll(() => page.evaluate(() => (window as unknown as { __FFMPEG_DOWNLOAD_URL__?: unknown }).__FFMPEG_DOWNLOAD_URL__))
      .toBe('https://apps.yuelili.com/software/ffmpeg');
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  });

  test(`settings leave guard and draft FFmpeg ${theme}`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await installTauriMock(page, theme);
    await page.addInitScript(() => {
      const target = window as unknown as {
        __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
        __FAIL_SAVE__?: boolean;
      };
      const invoke = target.__TAURI_INTERNALS__.invoke;
      target.__TAURI_INTERNALS__.invoke = async (command, args) => {
        if (command === 'settings_update' && target.__FAIL_SAVE__) throw new Error('测试保存失败');
        if (command === 'environment_health') {
          const result = await invoke(command, args) as { ffmpeg: Record<string, unknown> };
          const path = (args?.request as { ffmpeg_path?: string })?.ffmpeg_path;
          if (path) result.ffmpeg = { status: 'ready', source: 'configured', path, version: 'ffmpeg version custom', message: 'FFmpeg 可用。' };
          return result;
        }
        return invoke(command, args);
      };
    });
    await page.goto('/');
    await page.keyboard.press('Control+4');
    const path = page.getByRole('textbox', { name: 'FFmpeg 路径', exact: true, includeHidden: true });
    const custom = 'D:\\自定义工具\\' + 'long-directory-'.repeat(12) + 'ffmpeg.exe';
    await path.fill(custom);
    const summary = page.getByRole('region', { name: '运行环境' });
    await expect(summary).toContainText('未检查');
    await summary.getByRole('button', { name: '重新检查' }).click();
    await expect(summary).toContainText('指定路径：' + custom);
    await summary.scrollIntoViewIfNeeded();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: testInfo.outputPath('ffmpeg-draft-path.png'), animations: 'disabled' });
    await page.keyboard.press('Control+1');
    const dialog = page.getByRole('dialog', { name: '设置尚未保存' });
    await expect(dialog).toBeVisible();
    await page.setViewportSize({ width: 900, height: 700 });
    await page.screenshot({ path: testInfo.outputPath('settings-leave.png'), animations: 'disabled' });
    await dialog.getByRole('button', { name: '继续编辑' }).focus();
    await page.keyboard.press('Enter');
    await expect(dialog).not.toBeVisible();
    await expect(path).toHaveValue(custom);
    await page.keyboard.press('Control+1');
    await page.evaluate(() => { (window as unknown as { __FAIL_SAVE__: boolean }).__FAIL_SAVE__ = true; });
    await dialog.getByRole('button', { name: '保存并离开' }).click();
    await expect(dialog).toContainText('测试保存失败');
    await expect(path).toHaveValue(custom);
    await page.evaluate(() => { (window as unknown as { __FAIL_SAVE__: boolean }).__FAIL_SAVE__ = false; });
    await dialog.getByRole('button', { name: '保存并离开' }).click();
    await expect(dialog).not.toBeVisible();
    await expect(page.getByLabel('链接或 BV / AV')).toBeVisible();
    await page.keyboard.press('Control+4');
    await expect(path).toHaveValue(custom);
    await path.fill('D:\\discard.exe');
    await page.keyboard.press('Control+1');
    await dialog.getByRole('button', { name: '放弃更改' }).click();
    await page.keyboard.press('Control+4');
    await expect(path).toHaveValue(custom);
  });
}

for (const theme of ['light', 'dark'] as const) {
  test(`settings leave guard mobile ${theme}`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width: 375, height: 844 });
    await page.addInitScript(() => Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }));
    await installTauriMock(page, theme);
    await page.goto('/');
    const nav = page.getByRole('navigation', { name: '主导航' });
    await nav.getByRole('button', { name: '我的', exact: true }).click();
    await page.getByRole('button', { name: /下载设置/ }).click();
    await page.getByRole('button', { name: /下载 目录/ }).click();
    await page.getByRole('textbox', { name: '全局下载限速（MB/s）', exact: true }).fill('-1');
    await page.getByRole('button', { name: '返回设置', exact: true }).click();
    await page.getByRole('button', { name: '返回我的', exact: true }).click();
    const dialog = page.getByRole('dialog', { name: '设置尚未保存' });
    await expect(dialog).toBeVisible();
    await expect(dialog.getByRole('button', { name: '保存并离开' })).toBeDisabled();
    await page.screenshot({ path: testInfo.outputPath('mobile-settings-leave.png'), animations: 'disabled' });
    await page.keyboard.press('Escape');
    await expect(dialog).not.toBeVisible();
    await nav.getByRole('button', { name: '解析', exact: true }).click();
    await expect(dialog).toBeVisible();
    await dialog.getByRole('button', { name: '放弃更改' }).click();
    await expect(page.getByLabel('链接或 BV / AV')).toBeVisible();
  });
}

for (const theme of ['light', 'dark'] as const) {
  test(`Android update and transfer speed ${theme}`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width: 320, height: 844 });
    await page.addInitScript(() => Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }));
    await installTauriMock(page, theme);
    await page.route('https://api.github.com/repos/Yuelioi/bdl/releases/latest', (route) => route.fulfill({
      json: { tag_name: 'v0.8.3', draft: false, prerelease: false, body: '手机端更新与速度显示\nhttps://github.com/Yuelioi/bdl/' + 'long-path-'.repeat(32),
        assets: [{ name: 'BDL-v0.8.3-android-arm64.apk', size: 100, browser_download_url: 'https://github.com/Yuelioi/bdl/releases/download/v0.8.3/BDL-v0.8.3-android-arm64.apk' }] },
    }));
    await page.addInitScript(() => {
      const target = window as unknown as {
        __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown>; transformCallback: (callback: (event: unknown) => void) => number };
        __EMIT_MOBILE__: (event: string, payload: unknown) => void;
        __APK_URL__?: string;
      };
      const callbacks = new Map<number, (event: unknown) => void>();
      const listeners = new Map<string, number>();
      let id = 1000;
      target.__TAURI_INTERNALS__.transformCallback = (callback) => { callbacks.set(++id, callback); return id; };
      target.__EMIT_MOBILE__ = (event, payload) => callbacks.get(listeners.get(event) ?? -1)?.({ event, id: listeners.get(event), payload });
      const invoke = target.__TAURI_INTERNALS__.invoke;
      const task = { id: 'mobile-speed', title: '一个很长的下载任务标题，手机和平板都应显示速度与进度', source_id: 'video:fixture', status: 'downloading', output_path: 'downloads/test.mp4', refresh_intent: null, media_selection: null, scheduled_at: null, speed_limit_bytes_per_second: null,
        resources: [{ id: 'video', intent: 'video', status: 'downloading', current_urls: [], target_path: 'downloads/test.m4s' }] };
      target.__TAURI_INTERNALS__.invoke = async (command, args) => {
        if (command === 'plugin:app|version') return '0.8.2';
        if (command === 'plugin:event|listen') { listeners.set(args?.event as string, args?.handler as number); return id++; }
        if (command === 'queue_list') return [task];
        if (command === 'queue_logs') return [];
        if (command === 'open_external_url') { target.__APK_URL__ = args?.url as string; return null; }
        return invoke(command, args);
      };
    });
    await page.goto('/');
    const nav = page.getByRole('navigation', { name: '主导航' });
    await nav.getByRole('button', { name: '我的', exact: true }).click();
    await page.getByRole('button', { name: /下载设置/ }).click();
    await page.getByRole('button', { name: /应用更新/ }).click();
    await expect(page.getByRole('switch', { name: '自动检测更新' })).not.toBeChecked();
    await page.getByRole('button', { name: '检测更新', exact: true }).click();
    await expect(page.getByText('发现新版本 v0.8.3')).toBeVisible();
    await page.screenshot({ path: testInfo.outputPath('android-update.png'), animations: 'disabled' });
    await page.getByRole('button', { name: '下载 APK' }).click();
    await expect.poll(() => page.evaluate(() => (window as unknown as { __APK_URL__?: string }).__APK_URL__)).toBe('https://github.com/Yuelioi/bdl/releases/download/v0.8.3/BDL-v0.8.3-android-arm64.apk');
    await nav.getByRole('button', { name: /传输/ }).click();
    await page.evaluate(() => {
      const emit = (window as unknown as { __EMIT_MOBILE__: (event: string, payload: unknown) => void }).__EMIT_MOBILE__;
      const time = Date.now();
      for (const [downloaded, offset] of [[512 * 1024 * 1024, -1000], [1024 * 1024 * 1024, 0]]) emit('queue://progress-updated', { task_id: 'mobile-speed', resource_id: 'video', downloaded_bytes: downloaded, total_bytes: 4 * 1024 * 1024 * 1024, created_at: new Date(time + offset).toISOString() });
    });
    const meta = page.locator('.mobile-task .task-meta').first();
    await expect(meta).toContainText('MB/s');
    await expect(meta).toContainText('25%');
    for (const size of [{ width: 320, height: 844 }, { width: 568, height: 356 }, { width: 1280, height: 800 }]) {
      await page.setViewportSize(size);
      await expect(meta).toBeVisible();
      expect(await meta.evaluate((element) => element.scrollWidth <= element.clientWidth)).toBe(true);
      await page.screenshot({ path: testInfo.outputPath(`mobile-speed-${size.width}.png`), animations: 'disabled' });
    }
    await page.evaluate(() => {
      (window as unknown as { __EMIT_MOBILE__: (event: string, payload: unknown) => void }).__EMIT_MOBILE__('queue://task-updated', { id: 'mobile-speed', title: '已暂停', source_id: 'video:fixture', status: 'paused', resources: [], output_path: 'downloads/test.mp4', refresh_intent: null, media_selection: null, scheduled_at: null, speed_limit_bytes_per_second: null });
    });
    await expect(meta).not.toContainText('MB/s');
  });
}

test('desktop update notice remains separate from Android APK guidance', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.addInitScript(() => {
    const target = window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> } };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => command === 'plugin:updater|check'
      ? { rid: 99, currentVersion: '0.8.2', version: '0.8.3', body: '桌面更新' } : invoke(command, args);
  });
  await page.goto('/');
  await page.getByRole('button', { name: '设置 偏好与维护' }).click();
  await page.getByRole('button', { name: /应用更新/ }).click();
  await page.getByRole('button', { name: '检测更新', exact: true }).click();
  await expect(page.getByText('发现新版本 v0.8.3')).toBeVisible();
  await expect(page.getByRole('button', { name: '下载并安装' })).toBeVisible();
  await expect(page.getByRole('button', { name: '下载 APK' })).toHaveCount(0);
  await expect(page.getByText('当前已是最新版本。')).toHaveCount(0);
});
