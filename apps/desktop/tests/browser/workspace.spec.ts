// Browser behavior checks with mocked native APIs. Passing does not establish visual quality or native integration.
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
            if (command === 'selection_estimate_size') return { estimated_bytes: 0, estimated_parts: 0, unknown_streams: 1 };
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

test.describe('specific failure diagnostics', () => {
  for (const device of ['desktop', 'phone'] as const) {
    test(`${device} keeps source errors and diagnostic causes distinct`, async ({ page }) => {
      await page.setViewportSize(device === 'desktop' ? { width: 1280, height: 800 } : { width: 390, height: 844 });
      if (device === 'phone') await page.addInitScript(() => Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }));
      await installTauriMock(page, 'light', true, true);
      await page.addInitScript(() => {
        const target = window as unknown as {
          __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
          __COPIED_DIAGNOSTIC__: string;
          __EMPTY_LOGS__?: boolean;
        };
        Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: async (text: string) => { target.__COPIED_DIAGNOSTIC__ = text; } } });
        const task = { id: 'diagnostic-fixture', title: '源站拒绝访问测试', source_id: 'video:fixture', status: 'failed', resources: [], output_path: 'downloads/403.mp4', media_selection: null, scheduled_at: null, speed_limit_bytes_per_second: null };
        const invoke = target.__TAURI_INTERNALS__.invoke;
        target.__TAURI_INTERNALS__.invoke = async (command, args) => {
          if (command === 'diagnostics_environment') return { app_version: '0.8.4', os: 'android', arch: 'aarch64', device: { os_version: '16', android_sdk: 36, webview_version: '130.0' }, device_info_available: true, segment_count: 4, concurrent_tasks: 2, retry_count: 5, proxy_enabled: false };
          if (command === 'queue_list') return [task];
          if (command === 'queue_logs') return target.__EMPTY_LOGS__ ? [] : [
            { task_id: task.id, level: 'error', message: `[stage=download_video code=remote_forbidden] 请求资源长度失败: HTTP 403 Forbidden\n${'source=1 response cause; '.repeat(200)}\nsource=3 cause=last-source\nCookie: SESSDATA=private-cookie\nhttps://cdn.test/video?token=private-token`, created_at: '2026-10-14T12:02:00Z' },
            { task_id: task.id, level: 'info', message: '重试任务', created_at: '2026-10-14T12:01:00Z' },
            { task_id: task.id, level: 'error', message: 'Cookie 已失效', created_at: '2026-10-14T12:00:00Z' },
          ];
          return invoke(command, args);
        };
      });
      await page.goto('/');
      await page.getByRole('button', { name: /传输/ }).click();
      const row = page.getByRole('list', { name: '传输任务', includeHidden: true }).getByRole('listitem', { includeHidden: true }).filter({ hasText: '源站拒绝访问测试' });
      await expect(row).toContainText('源站拒绝访问');
      await row.getByRole('button', { name: /更多操作/ }).click();
      await page.getByRole(device === 'desktop' ? 'menuitem' : 'button', { name: '详情和诊断', exact: true }).click();
      const dialog = page.getByRole('dialog', { name: '源站拒绝访问测试', exact: true });
      await expect(dialog.getByRole('heading', { name: '任务失败：源站拒绝访问' })).toBeVisible();
      await expect(dialog).toContainText('单凭 403 无法判断');
      await dialog.getByRole('button', { name: '复制诊断信息', exact: true }).click();
      const copied = await page.evaluate(() => (window as unknown as { __COPIED_DIAGNOSTIC__: string }).__COPIED_DIAGNOSTIC__);
      expect(copied).toContain('HTTP 403 Forbidden');
      expect(copied).toContain('stage=download_video');
      expect(copied).toContain('source=3 cause=last-source');
      expect(copied).toContain('系统: android 16 · aarch64');
      expect(copied).toContain('分段 4 · 并发 2 · 重试 5');
      expect(copied).not.toContain('private-cookie');
      expect(copied).not.toContain('private-token');
      await dialog.getByRole('tab', { name: /原始日志/ }).click();
      await dialog.getByRole('button', { name: '一键复制', exact: true }).click();
      await expect(dialog.getByRole('button', { name: '已复制', exact: true })).toBeVisible();
      const copiedLogs = await page.evaluate(() => (window as unknown as { __COPIED_DIAGNOSTIC__: string }).__COPIED_DIAGNOSTIC__);
      expect(copiedLogs).toContain('任务 ID: diagnostic-fixture');
      expect(copiedLogs).toContain('系统: android 16 · aarch64');
      expect(copiedLogs).toContain('[error] 2026-10-14T12:02:00Z');
      expect(copiedLogs).toContain('HTTP 403 Forbidden');
      expect(copiedLogs).toContain('source=3 cause=last-source');
      expect(copiedLogs).not.toContain('诊断:');
      expect(copiedLogs).not.toContain('private-cookie');
      expect(copiedLogs).not.toContain('private-token');
      await page.evaluate(() => Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: async () => { throw new Error('clipboard unavailable'); } } }));
      await dialog.getByRole('button', { name: /^(已复制|一键复制)$/ }).click();
      await expect(page.getByText('复制失败，请重试', { exact: true })).toBeVisible();
      await expect(dialog.getByRole('button', { name: '一键复制', exact: true })).toBeVisible();
      await page.addInitScript(() => { (window as unknown as { __EMPTY_LOGS__: boolean }).__EMPTY_LOGS__ = true; });
      await page.reload();
      await page.getByRole('button', { name: /传输/ }).click();
      await row.getByRole('button', { name: /更多操作/ }).click();
      await page.getByRole(device === 'desktop' ? 'menuitem' : 'button', { name: '详情和诊断', exact: true }).click();
      await dialog.getByRole('tab', { name: /原始日志/ }).click();
      await expect(dialog.getByRole('button', { name: '一键复制', exact: true })).toBeDisabled();
    });
  }
});

test.describe('safe file removal', () => {
  test.use({ deviceScaleFactor: 2.25 });
  for (const theme of ['light', 'dark'] as const) {
    for (const device of ['desktop', 'tablet', 'phone'] as const) {
      test(`${theme} ${device} previews explicit selection and confirms deletion`, async ({ page }) => {
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
              if (!failedOnce) { failedOnce = true; throw { code: 'file_in_use', message: '文件被占用，请关闭播放器后重新确认', detail: '[stage=file_cleanup code=file_in_use] 打开待删除文件失败：C:/downloads/selected.mp4；os_error=32\nCookie: SESSDATA=private-cookie' }; }
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
        await page.locator('.bulk-action-bar').getByRole('button', { name: '删除任务及文件', exact: true }).click();
        await expect(dialog.getByText(/1 个成品或附件/)).toBeVisible();
        await dialog.getByText('查看将删除的文件', { exact: true }).click();
        await expect(dialog.getByText(/selected.mp4/, { exact: false })).toBeVisible();
        await dialog.getByRole('button', { name: '确认删除任务及文件', exact: true }).click();
        await expect(dialog.getByText(/文件被占用/)).toBeVisible();
        await dialog.getByText('错误详情', { exact: true }).click();
        await expect(dialog.getByText(/os_error=32/)).toBeVisible();
        await expect(dialog).not.toContainText('private-cookie');
        await expect(dialog.getByRole('button', { name: '复制错误信息', exact: true })).toBeVisible();
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

test('mobile shell keeps navigation and downloads usable without desktop steps', async ({ page }) => {
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
  await expect(page.locator('.mobile-header .account-button')).toHaveCount(0);
  await expect(page.locator('.mobile-header').getByRole('button')).toHaveCount(0);
  await page.getByLabel('Bilibili 链接或 BV / AV').fill('favorite:fixture');
  await page.getByRole('button', { name: '开始解析' }).click();
  const results = page.getByRole('list', { name: '解析结果' });
  await expect(results).toBeVisible();
  const parseRow = results.getByRole('listitem').first();
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
  }
  await page.getByRole('button', { name: '下载所选 (1)' }).click();
  const downloadDialog = page.getByRole('dialog');
  await expect(downloadDialog).toBeVisible();
});

test.describe('adaptive Android workspace', () => {
  test.use({ deviceScaleFactor: 2.25 });
  for (const theme of ['light', 'dark'] as const) {
    test(`${theme} navigation preserves selection on rotation`, async ({ page }) => {
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
      const searchButton = page.getByRole('button', { name: '搜索视频', exact: true });
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
      await page.getByRole('button', { name: '关闭搜索', exact: true }).click();
      await expect(results.getByRole('listitem')).toHaveCount(40);
      await page.getByRole('button', { name: '完成管理', exact: true }).click();
      await expect(results.getByRole('checkbox')).toHaveCount(0);
      await expect(page.getByRole('button', { name: '下载所选 (0)', exact: true })).toBeDisabled();
      await page.getByRole('button', { name: '内容操作', exact: true }).click();
      const contentPanel = page.getByRole('dialog', { name: '内容操作' });
      await expect(contentPanel.locator('.tablet-action-body')).toBeVisible();
      await page.getByLabel('每次加载').selectOption('100');
      await page.setViewportSize({ width: 390, height: 844 });
      await expect(contentPanel.locator('.mobile-sheet-body')).toBeVisible();
      await expect(page.getByLabel('每次加载')).toHaveValue('100');
      await page.setViewportSize({ width: 568, height: 356 });
      await expect(contentPanel.locator('.tablet-action-body')).toBeVisible();
      await expect(page.getByLabel('每次加载')).toHaveValue('100');
      await page.keyboard.press('Escape');
      await expect(contentPanel).not.toBeVisible();
      await results.getByRole('listitem').first().locator('.mobile-result-content').click();
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
        await expect(download).toBeEnabled();
      }
      await download.click();
      const dialog = page.getByRole('dialog', { name: '下载设置' });
      const preset = dialog.getByRole('combobox', { name: /^下载预设/ });
      await preset.click();
      const option = page.getByRole('option', { name: '快速下载', exact: true });
      await option.click();
      await expect(preset).toContainText('快速下载');
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
  test(`tablet dialogs ${theme} preserve form input and restore focus after rotation`, async ({ page }) => {
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
      await dialog.getByRole('tab', { name: '下载', exact: true }).click();
      const scrollArea = dialog.locator('.download-options-scroll');
      await scrollArea.evaluate((element) => {
        element.scrollTop = element.scrollHeight;
      });
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
  test(`tablet dialogs ${theme} preset editor keeps controls and draft on rotation`, async ({ page }) => {
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
    }
    await editor.getByRole('button', { name: '取消', exact: true }).click();
    await expect(editor).not.toBeVisible();
    await expect(trigger).toBeFocused();
  });
}

test('mobile library supports selection and page actions', async ({ page }) => {
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
  await expect(folderRow.getByRole('checkbox')).toHaveCount(0);
  await page.getByRole('button', { name: '管理视频', exact: true }).click();
  await expect(folderRow.getByRole('checkbox')).toBeVisible();
  await page.locator('.mobile-result-content').first().click();
  await expect(page.locator('.folder-download').getByRole('button', { name: '下载所选 (1)' })).toBeEnabled();
  for (const width of [390, 320]) {
    await page.setViewportSize({ width, height: 844 });
  }
  await page.locator('.folder-download').getByRole('button').click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.keyboard.press('Escape');
  await nav.getByRole('button', { name: '我的' }).click();
  await expect(page.locator('.profile-card')).toBeVisible();
  await expect(page.locator('.workspace-enter-active, .workspace-leave-active')).toHaveCount(0);
  await page.getByRole('button', { name: /下载设置/ }).click();
  await page.getByRole('button', { name: '下载 目录、速度与任务恢复' }).click();
  await page.getByLabel('全局下载限速（MB/s）', { exact: true }).fill('10');
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByRole('button', { name: '保存', exact: true })).toBeDisabled();
  await nav.getByRole('button', { name: '传输' }).click();
  await expect(page.locator('.transfer-main')).toBeVisible();
  await nav.getByRole('button', { name: '我的' }).click();
  await page.getByRole('button', { name: /关于 BDL/ }).click();
  await expect(page.getByText('GitHub 仓库')).toBeVisible();
});

test.describe('populated Android interactions', () => {
  test.use({ deviceScaleFactor: 2.25 });

  test('Android failed task offers a visible retry action in its menu', async ({ page }) => {
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
    await retry.click();
    await expect(menu).toHaveCount(0);
    await expect
      .poll(() => page.evaluate(() => (window as unknown as { __RETRY_REQUESTS__: unknown[] }).__RETRY_REQUESTS__))
      .toEqual([{ taskId: 'failed-media' }]);
    await expect(row).toHaveCount(0);
    await expect(page.getByRole('alert')).toHaveCount(0);
  });

  test('mobile menus, list virtualization and settings navigation work with populated data', async ({ page }) => {
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
    await expect(page.getByRole('button', { name: '开始解析' })).toBeVisible();
    const nav = page.getByRole('navigation', { name: '主导航' });
    await nav.getByRole('button', { name: '内容库' }).click();
    const firstCollection = page.locator('.library-card').first();
    await expect(firstCollection.locator('.library-description')).toHaveCount(0);
    await expect(firstCollection.locator('.library-card-copy').locator(':scope > *')).toHaveCount(2);
    await page.setViewportSize({ width: 568, height: 356 });
    await page.setViewportSize({ width: 390, height: 844 });
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
    }
    await page.getByRole('button', { name: '内容操作' }).click();
    await expect(page.getByRole('dialog', { name: '内容操作' })).toBeVisible();
    await expect(page.getByLabel('每次加载')).toBeVisible();
    await page.keyboard.press('Escape');
    await nav.getByRole('button', { name: /传输/ }).click();
    await expect(page.getByRole('list', { name: '传输任务' })).toBeVisible();
    await expect(page.getByRole('table', { name: '传输任务' })).toHaveCount(0);
    await expect(page.locator('.task-playback-action').first()).toBeVisible();
    await expect(page.locator('.mobile-task').first()).toContainText('失败');
    await expect(page.locator('.mobile-task').filter({ hasText: '下载视频中' })).toHaveCount(1);
    expect(
      await page
        .locator('.task-playback-action')
        .first()
        .locator('svg')
        .evaluate((icon) => icon.childElementCount),
    ).toBeGreaterThan(0);
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
    await expect(page.locator('.mobile-header')).toContainText('BDL');
    const firstTask = page.locator('.mobile-task').first();
    await firstTask.hover();
    await page.mouse.down();
    await page.waitForTimeout(460);
    await page.mouse.up();
    await expect(firstTask.getByRole('checkbox')).toBeChecked();
    await expect(page.getByRole('button', { name: '完成管理' })).toBeVisible();
    await page.getByRole('button', { name: '完成管理' }).click();
    await expect(firstTask.getByRole('checkbox')).toHaveCount(0);
    await page
      .getByRole('button', { name: /：更多操作/ })
      .first()
      .click();
    await expect(page.getByRole('dialog', { name: '任务操作' })).toBeVisible();
    await page.keyboard.press('Escape');
    await page.getByRole('button', { name: '传输选项' }).click();
    await expect(page.getByRole('dialog', { name: '传输选项' })).toBeVisible();
    await page.keyboard.press('Escape');
    // Exercise virtualization with both row sizes, including its final window.
    const tasks = page.getByRole('list', { name: '传输任务' });
    for (const viewport of [
      { width: 568, height: 356, row: 64 },
      { width: 390, height: 844, row: 62 },
    ]) {
      await page.setViewportSize({ width: viewport.width, height: viewport.height });
      if (viewport.width === 568) {
        await expect(tasks.getByRole('listitem').first().locator('.task-meta')).toContainText('已完成');
        await expect(tasks.getByRole('listitem').first().locator('.task-meta')).toContainText('1:00');
        await expect(
          tasks.getByRole('listitem').first().getByRole('button', { name: '播放', exact: true }),
        ).toBeVisible();
      }
      await tasks.evaluate((list) => {
        list.scrollTop = list.scrollHeight;
      });
      expect(await tasks.getByRole('listitem').count()).toBeLessThan(40);
    }
    await nav.getByRole('button', { name: '我的' }).click();
    await expect(page.locator('.profile-card')).toBeVisible();
    await expect(page.locator('.workspace-enter-active, .workspace-leave-active')).toHaveCount(0);
    await page.getByRole('button', { name: /下载设置/ }).click();
    await expect(page.locator('.settings-category')).toHaveCount(5);
    await expect(page.locator('.mobile-header')).toBeVisible();
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
    await page.getByRole('button', { name: /下载设置/ }).click();
    await nav.getByRole('button', { name: /传输/ }).click();
    await expect(page.locator('.mobile-task-list')).toBeVisible();
    await expect(page.locator('.workspace-enter-active, .workspace-leave-active')).toHaveCount(0);
    await expect(page.getByRole('alert')).toHaveCount(0);
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
  test(`appearance hover labels follow theme selection at ${width}px`, async ({ page }) => {
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
    }
  });
}

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

test('single video mode uses one input and disables collection expansion', async ({ page }) => {
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
  await page.setViewportSize({ width: 540, height: 720 });
  await expect(input).toBeVisible();
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

test('parse result checkbox controls download selection', async ({ page }) => {
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

test('parse cooldown shows remaining seconds and stops immediately', async ({ page }) => {
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
  await toolbar.getByRole('button', { name: '停止解析', exact: true }).click();
  await expect(toolbar.getByRole('button', { name: '解析', exact: true })).toBeVisible({ timeout: 1000 });
  await expect(toolbar).toContainText('9 / 401');
  await expect(page.getByText('解析已停止', { exact: true })).toHaveCount(0);
});

test('parse all can stop at a page boundary', async ({ page }) => {
  const toolbar = page.locator('.source-function-toolbar');
  await installTauriMock(page, 'light');
  await page.goto('/');
  await page.getByLabel('链接或 BV / AV').fill('favorite:fixture');
  await page.getByRole('button', { name: '开始解析' }).click();
  await expect(toolbar.getByRole('combobox', { name: '每批解析数量' })).toContainText('每批 50');
  await toolbar.getByRole('button', { name: '更多解析方式' }).click();
  await page.getByRole('menuitem', { name: '解析全部', exact: true }).click();
  await toolbar.getByRole('button', { name: '停止解析', exact: true }).click();
  await expect(toolbar.getByRole('button', { name: '解析', exact: true })).toBeVisible();
});

for (const theme of ['light', 'dark'] as const) {
  for (const platform of ['desktop', 'mobile'] as const) {
  test(`background parsing failure recovery ${platform} ${theme}`, async ({ page }) => {
    if (platform === 'mobile') {
      await page.setViewportSize({ width: 375, height: 844 });
      await page.addInitScript(() => Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }));
    }
    await installTauriMock(page, theme);
    await page.addInitScript(() => {
      const target = window as unknown as {
        __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
        __FAILED_PART__?: string;
        __RETRY_ALLOWED__?: boolean;
        __TASK_ATTEMPTS__: string[];
      };
      target.__TASK_ATTEMPTS__ = [];
      const invoke = target.__TAURI_INTERNALS__.invoke;
      target.__TAURI_INTERNALS__.invoke = async (command, args) => {
        if (command === 'selection_create_tasks') {
          const id = (args?.request as { part_ids: string[] }).part_ids[0];
          target.__TASK_ATTEMPTS__.push(id);
          target.__FAILED_PART__ ??= id;
          if (id === target.__FAILED_PART__ && !target.__RETRY_ALLOWED__) throw new Error('planning error: `长标题' + '测试'.repeat(70) + '` 缺少视频流，请重新解析后再试。');
        }
        return invoke(command, args);
      };
    });
    await page.goto('/');
    await page.getByLabel('链接或 BV / AV').fill('favorite:fixture');
    await page.getByRole('button', { name: '开始解析' }).click();
    if (platform === 'mobile') {
      await page.getByRole('button', { name: '内容操作', exact: true }).click();
      await page.getByRole('dialog', { name: '内容操作' }).getByRole('button', { name: '下载整个集合', exact: true }).click();
    } else {
      await page.getByRole('button', { name: '更多解析方式' }).click();
      await page.getByRole('menuitem', { name: '后台解析全部并下载' }).click();
    }
    const status = page.getByRole('region', { name: '后台解析下载' });
    await expect(status).toContainText('后台解析下载已完成', { timeout: 15000 });
    await expect(status).toContainText('1 项失败');
    const before = await page.evaluate(() => (window as unknown as { __TASK_ATTEMPTS__: string[] }).__TASK_ATTEMPTS__.length);
    expect(before).toBeGreaterThan(1);
    await status.getByText('1 项失败，展开查看', { exact: true }).focus();
    await page.keyboard.press('Enter');
    for (const width of [1280, 375, 568]) {
      await page.setViewportSize({ width, height: width === 568 ? 356 : 844 });
    }
    await page.evaluate(() => { (window as unknown as { __RETRY_ALLOWED__: boolean }).__RETRY_ALLOWED__ = true; });
    await status.getByRole('button', { name: '重试失败项' }).focus();
    await page.keyboard.press('Enter');
    await expect(status).toContainText('后台解析下载已完成');
    await expect(status.getByRole('button', { name: '重试失败项' })).not.toBeVisible();
    expect(await page.evaluate(() => (window as unknown as { __TASK_ATTEMPTS__: string[] }).__TASK_ATTEMPTS__.length)).toBe(before + 1);
  });
  }
}

test('background parsing downloads progressively and remains visible on another page', async ({ page }) => {
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

test('library omits redundant category headings and makes the whole folder card the enter action', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await installTauriMock(page, 'light', true, true);
  await page.goto('/');
  await page.getByRole('button', { name: '内容库 收藏与订阅' }).click();

  const categoryLink = page.getByRole('button', { name: '在 Bilibili 打开我的收藏夹' });
  await expect(categoryLink).toHaveCount(0);
  await expect(page.locator('.library-header')).toHaveCount(0);
  await expect(page.getByRole('button', { name: '在 Bilibili 打开 Web' })).toHaveCount(0);
  const enterFolder = page.getByRole('button', { name: '进入 Web' });
  await expect(enterFolder.locator('.ui-icon-button')).toHaveCount(0);
  await expect(enterFolder.locator('.library-description')).toHaveCount(0);
  await expect(enterFolder.locator('.library-card-meta')).toContainText('个视频');

  await enterFolder.click();
  await expect(page.getByRole('button', { name: '返回内容集合' })).toBeVisible();
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
  await expect(dialog.locator('.inline-notice > p').filter({ hasText: 'Bilibili 暂时拒绝了请求' })).toBeVisible();
  await expect(dialog.getByRole('button', { name: '复制诊断信息' })).toBeVisible();
  await dialog.getByText('错误详情', { exact: true }).click();
  await expect(dialog.locator('.error-details pre')).toContainText('bilibili_request_rejected');

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

test('library detail supports pagination, parsing and selection', async ({
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
});

test('library page jump loads only the requested page and reuses visited pages', async ({ page }) => {
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
  await page.setViewportSize({ width: 540, height: 720 });
  await expect(pagination.getByRole('textbox')).toBeVisible();
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

test('library video rows toggle selection and support select-all', async ({ page }) => {
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
  await firstCard.getByRole('checkbox').click();
  await page.getByRole('checkbox', { name: '全选本页' }).click();
  await expect(page.getByRole('checkbox', { name: '全选本页' })).toBeChecked();
});

test('media preferences reorder combinations while download keeps optimal quality', async ({ page }) => {
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
  await page.setViewportSize({ width: 800, height: 900 });
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

test('download tabs use saved presets and keep overrides local', async ({ page }) => {
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
  await dialog.getByRole('tab', { name: '调度', exact: true }).click();
  await page.setViewportSize({ width: 900, height: 600 });
  await dialog.getByRole('tab', { name: '下载', exact: true }).click();
  const scrollArea = dialog.locator('.download-options-scroll');
  await scrollArea.evaluate((element) => {
    element.scrollTop = element.scrollHeight;
  });
  await expect.poll(() => scrollArea.evaluate((element) => element.scrollTop)).toBeGreaterThan(0);
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

test('saved naming presets are reusable while unsaved defaults stay in settings', async ({ page }) => {
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

test('download archive settings inherit defaults and override processing per task', async ({ page }) => {
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
  test(`settings cleanup ${theme} desktop media and direct attachments`, async ({ page }) => {
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

  test(`settings cleanup ${theme} one-time download options`, async ({ page }) => {
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
    await dialog.getByRole('button', { name: '开始下载', exact: true }).click();
    await expect
      .poll(() => page.evaluate(() => (window as unknown as { __CLEAN_REQUEST__?: unknown }).__CLEAN_REQUEST__))
      .toMatchObject({ download_preset_id: 'mkv' });
  });
}

for (const theme of ['light', 'dark'] as const) {
  test(`settings cleanup ${theme} mobile attachments and MKV`, async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 844 });
    await page.addInitScript(() =>
      Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }),
    );
    await installTauriMock(page, theme);
    await installPresetPersistence(page);
    await page.goto('/');
    await openPresetSettings(page);
    await expect(page.getByRole('checkbox')).toHaveCount(0);
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
    await editor.getByRole('button', { name: '保存预设', exact: true }).click();
    await expect(editor).not.toBeVisible();
    await page.getByRole('button', { name: '返回设置', exact: true }).click();
    await page.getByRole('button', { name: /下载 目录/ }).click();
    await expect(page.getByRole('combobox', { name: '单任务分段数', exact: true })).toBeVisible();
    await expect(page.getByRole('textbox', { name: 'FFmpeg 路径', exact: true })).toHaveCount(0);
  });
}

test('pagination presets save rules and allow custom cooldown', async ({ page }) => {
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
  }) => {
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
    await page.mouse.move(0, 0);
    await page.getByRole('button', { name: '内容库 收藏与订阅' }).click();
    await page.getByRole('tab', { name: /订阅合集/ }).click();
    await page.getByRole('button', { name: '进入 双赢之路' }).click();
    await expect(page.locator('.library-panel').getByRole('tablist')).toHaveCount(0);
    const libraryRow = page.locator('.source-media-row').first();
    await expect(libraryRow.locator('.media-title')).toHaveText(longTitle);
    await libraryRow.getByRole('button', { name: `播放 ${longTitle}`, exact: true }).click();
    await expect(libraryRow.getByRole('checkbox')).not.toBeChecked();
    await page.keyboard.press('Control+3');
    await page.getByRole('tab', { name: /全部/ }).click();
    await expect(page.locator('.task-table-row')).toHaveCount(6);
    await expect(page.locator('.task-table-row').first().locator('.media-size')).toContainText('128 MB');
    await page.locator('.task-table-row').first().getByRole('button', { name: '更多操作' }).click();
    await expect(page.getByRole('menuitem', { name: '打开文件夹' })).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(page.getByRole('menuitem', { name: '打开文件夹' })).toBeHidden();
    await page.setViewportSize({ width: 900, height: 700 });
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
});

for (const [format, width, theme, danmakuFormat] of [
  ['m4s', 1280, 'light', 'HTML（离线播放）'],
  ['mp3', 900, 'dark', 'SRT（字幕显示）'],
  ['mp3', 375, 'light', 'ASS（滚动弹幕）'],
] as const) {
  test(`download recipe remembers ${format} at ${width}px`, async ({ page }) => {
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
});

for (const theme of ['light', 'dark'] as const) {
  test(`missing FFmpeg download link ${theme}`, async ({ page }) => {
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
    await download.click();
    await expect.poll(() => page.evaluate(() => (window as unknown as { __FFMPEG_DOWNLOAD_URL__?: unknown }).__FFMPEG_DOWNLOAD_URL__))
      .toBe('https://apps.yuelili.com/software/ffmpeg');
  });

  test(`settings leave guard and draft FFmpeg ${theme}`, async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await installTauriMock(page, theme);
    await page.addInitScript(() => {
      const target = window as unknown as {
        __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
        __FAIL_SAVE__?: boolean;
        __FFMPEG_HELP_URL__?: string;
      };
      const invoke = target.__TAURI_INTERNALS__.invoke;
      target.__TAURI_INTERNALS__.invoke = async (command, args) => {
        if (command === 'settings_update' && target.__FAIL_SAVE__) throw new Error('测试保存失败');
        if (command === 'open_external_url') { target.__FFMPEG_HELP_URL__ = args?.url as string; return; }
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
    const help = page.getByText('如何配置 FFmpeg', { exact: true });
    await help.focus();
    await page.keyboard.press('Enter');
    const helpPanel = page.locator('details').filter({ has: help });
    await expect(helpPanel).toContainText('系统 PATH');
    await helpPanel.getByRole('button', { name: '下载 FFmpeg', exact: true }).click();
    await expect.poll(() => page.evaluate(() => (window as unknown as { __FFMPEG_HELP_URL__: string }).__FFMPEG_HELP_URL__)).toBe('https://apps.yuelili.com/software/ffmpeg');
    for (const width of [1280, 375]) {
      await page.setViewportSize({ width, height: 900 });
      await helpPanel.scrollIntoViewIfNeeded();
    }
    await page.setViewportSize({ width: 1280, height: 900 });
    await help.click();
    const path = page.getByRole('textbox', { name: 'FFmpeg 路径', exact: true, includeHidden: true });
    const custom = 'D:\\自定义工具\\' + 'long-directory-'.repeat(12) + 'ffmpeg.exe';
    await path.fill(custom);
    const summary = page.getByRole('region', { name: '运行环境' });
    await expect(summary).toContainText('未检查');
    await summary.getByRole('button', { name: '重新检查' }).click();
    await expect(summary).toContainText('指定路径：' + custom);
    await summary.scrollIntoViewIfNeeded();
    await page.keyboard.press('Control+1');
    const dialog = page.getByRole('dialog', { name: '设置尚未保存' });
    await expect(dialog).toBeVisible();
    await page.setViewportSize({ width: 900, height: 700 });
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
  test(`settings leave guard mobile ${theme}`, async ({ page }) => {
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
    await page.keyboard.press('Escape');
    await expect(dialog).not.toBeVisible();
    await nav.getByRole('button', { name: '解析', exact: true }).click();
    await expect(dialog).toBeVisible();
    await dialog.getByRole('button', { name: '放弃更改' }).click();
    await expect(page.getByLabel('链接或 BV / AV')).toBeVisible();
  });
}

for (const theme of ['light', 'dark'] as const) {
  test(`Android update and transfer speed ${theme}`, async ({ page }) => {
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

for (const [width, theme] of [[390, 'light'], [1280, 'dark'], [390, 'dark'], [1280, 'light']] as const) {
  test(`update source selection ${width} ${theme}`, async ({ page }) => {
    await page.setViewportSize({ width, height: 850 });
    if (width === 390) await page.addInitScript(() => Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }));
    await installTauriMock(page, theme);
    await page.addInitScript(() => {
      const target = window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> }; __UPDATE_TEMPLATE__?: string; __UPDATE_URL__?: string };
      const original = target.__TAURI_INTERNALS__.invoke;
      target.__TAURI_INTERNALS__.invoke = async (command, args) => {
        if (command === 'plugin:app|version') return '0.8.5';
        if (command === 'desktop_read_clipboard_text' || command === 'mobile_read_clipboard_text') return '';
        if (command === 'update_distribution') return {
          release: { version: '99.0.0', fileName: 'BDL.exe' }, releaseNotes: '多来源更新测试',
          sources: [
            { id: 'github', kind: 'github', label: 'GitHub', delivery: { mode: 'file', authenticationRequired: false, url: 'https://github.com/Yuelioi/bdl/releases/download/v99.0.0/BDL.exe' } },
            { id: 'quark', kind: 'cloud_drive', label: '夸克', delivery: { mode: 'browser', authenticationRequired: false, url: 'https://pan.quark.cn/s/test' } },
          ],
        };
        if (command === 'update_check_mirror') {
          target.__UPDATE_TEMPLATE__ = (args as { template: string }).template;
          return { rid: 99, currentVersion: '0.8.5', version: '99.0.0', body: '镜像更新', rawJson: {} };
        }
        if (command === 'open_external_url') { target.__UPDATE_URL__ = (args as { url: string }).url; return; }
        return original(command, args);
      };
    });
    await page.goto('/');
    if (width === 390) {
      await page.getByRole('navigation', { name: '主导航' }).getByRole('button', { name: '我的', exact: true }).click();
      await page.getByRole('button', { name: /下载设置/ }).click();
    } else await page.getByRole('button', { name: /^设置/ }).click();
    await page.getByRole('button', { name: /网络与维护/ }).click();
    const clipboard = page.getByRole('checkbox', { name: '返回应用时识别剪贴板链接' });
    await expect(clipboard).not.toBeChecked();
    await clipboard.focus();
    await page.keyboard.press('Space');
    await expect(clipboard).toBeChecked();
    await clipboard.uncheck();
    await expect(clipboard).not.toBeChecked();
    if (width === 390) await page.getByRole('button', { name: '返回设置', exact: true }).click();
    await page.getByRole('button', { name: /应用更新/ }).click();
    await page.getByRole('combobox', { name: '更新来源' }).click();
    await page.getByRole('option', { name: 'GitHub 加速下载' }).click();
    await page.getByLabel('自定义加速地址').fill('https://mirror.example/{url}');
    await page.getByRole('button', { name: '检测更新', exact: true }).click();
    await expect(page.getByText('发现新版本 v99.0.0')).toBeVisible();
    await expect(page.getByRole('button', { name: width === 390 ? '下载 APK' : '下载并安装' })).toBeVisible();
    if (width !== 390) await expect.poll(() => page.evaluate(() => (window as unknown as { __UPDATE_TEMPLATE__: string }).__UPDATE_TEMPLATE__)).toBe('https://mirror.example/{url}');
    await page.getByRole('combobox', { name: '更新来源' }).focus();
    await page.keyboard.press('Enter');
    await page.getByRole('option', { name: '夸克手动下载' }).click();
    await expect(page.getByText('发现新版本 v99.0.0')).toHaveCount(0);
    await page.getByRole('button', { name: '打开夸克网盘' }).focus();
    await page.keyboard.press('Enter');
    await expect.poll(() => page.evaluate(() => (window as unknown as { __UPDATE_URL__: string }).__UPDATE_URL__)).toBe('https://pan.quark.cn/s/test');
  });
}

for (const [width, theme] of [[390, 'light'], [1280, 'dark']] as const) {
  test(`NFO preset configuration ${width} ${theme}`, async ({ page }) => {
    await page.setViewportSize({ width, height: 850 });
    await installTauriMock(page, theme);
    await installPresetPersistence(page);
    await page.goto('/');
    await openPresetSettings(page);
    await page.getByRole('button', { name: '编辑', exact: true }).click();
    const editor = page.getByRole('dialog', { name: '编辑预设', exact: true });
    await editor.getByRole('checkbox', { name: '生成 NFO', exact: true }).check();
    await editor.getByRole('tab', { name: '转换', exact: true }).click();
    await editor.getByLabel('NFO 标题模板', { exact: true }).fill('{video_title} - {part_title}');
    await editor.getByRole('textbox', { name: /^自定义标签/ }).fill('教程, 收藏');
    await editor.getByRole('checkbox', { name: '将 UP 主加入人物列表' }).check();
    await editor.getByText('进阶字段：排序、作品集与制作信息', { exact: true }).click();
    await editor.getByRole('textbox', { name: '排序标题', exact: true }).fill('排序：');
    await editor.getByRole('button', { name: '排序标题：插入变量', exact: true }).focus();
    await page.keyboard.press('Enter');
    await page.locator('[data-variable="part_index:03"]').click();
    await page.keyboard.press('Escape');
    await expect(page.locator('.naming-variable-menu')).toHaveCount(0);
    await expect(editor.getByRole('textbox', { name: '排序标题', exact: true })).toHaveValue('排序：{part_index:03}');
    await editor.getByRole('textbox', { name: '作品集名称', exact: true }).fill('{video_title}');
    await editor.getByText('进阶字段：排序、作品集与制作信息', { exact: true }).click();
    await editor.getByText('模板变量与示例', { exact: true }).click();
    await expect(editor.getByText('标题示例：示例视频 - 开场', { exact: false }).first()).toBeVisible();
    await editor.getByRole('button', { name: '保存预设', exact: true }).click();
    await page.getByRole('button', { name: '编辑', exact: true }).click();
    await editor.getByRole('tab', { name: '转换', exact: true }).click();
    await expect(editor.getByRole('textbox', { name: /^自定义标签/ })).toHaveValue('教程, 收藏');
    await expect(editor.getByRole('checkbox', { name: '将 UP 主加入人物列表' })).toBeChecked();
    await editor.getByText('进阶字段：排序、作品集与制作信息', { exact: true }).click();
    await expect(editor.getByRole('textbox', { name: '排序标题', exact: true })).toHaveValue('排序：{part_index:03}');
    await editor.getByText('进阶字段：排序、作品集与制作信息', { exact: true }).click();
    await editor.getByText('套用展示方案', { exact: true }).click();
    await editor.getByRole('combobox', { name: 'NFO 展示方案', exact: true }).click();
    await page.getByRole('option', { name: '多 P 视频归组', exact: true }).click();
    await editor.getByRole('button', { name: '套用方案', exact: true }).click();
    await expect(editor.getByLabel('NFO 标题模板', { exact: true })).toHaveValue('{video_title} - P{part_index} - {part_title}');

    await editor.getByLabel('NFO 标题模板', { exact: true }).fill('{unknown}');
    await expect(editor.getByRole('button', { name: '保存预设', exact: true })).toBeDisabled();
  });
}

for (const device of ['desktop', 'android'] as const) {
  test(`issue10 watch later, card selection and clipboard on ${device}`, async ({ page }) => {
    await page.setViewportSize(device === 'desktop' ? { width: 1280, height: 850 } : { width: 390, height: 844 });
    if (device === 'android') await page.addInitScript(() => Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }));
    await installTauriMock(page, 'light', true, true);
    await page.addInitScript(() => {
      Object.assign(window, { isTauri: true, __ISSUE10_CLIP__: 'https://www.bilibili.com/video/BV1xx411c7mD' });
      Object.assign(window, { __WATCH_DOWNLOADS__: [] });
      const target = window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> } };
      const original = target.__TAURI_INTERNALS__.invoke;
      target.__TAURI_INTERNALS__.invoke = async (command, args) => {
        if (command.endsWith('read_clipboard_text')) return (window as unknown as { __ISSUE10_CLIP__: string }).__ISSUE10_CLIP__;
        if (command === 'settings_get') return { ...await original(command, args) as object, document_tree_output: { tree_uri: 'content://fixture/downloads', display_name: '下载' } };
        if (command === 'selection_estimate_size') return { estimated_bytes: 1000, estimated_parts: 20, unknown_streams: 0 };
        if (command === 'selection_create_tasks') {
          (window as unknown as { __WATCH_DOWNLOADS__: unknown[] }).__WATCH_DOWNLOADS__.push(args);
          return { created: [], duplicates: [], skipped_existing: 0, requires_confirmation: false };
        }
        if (command === 'parse_create_source' && (args as { request: { input: string } }).request.input.includes('watchlater')) return {
          source: { id: 'watch_later:test', kind: 'watch_later', input: 'https://www.bilibili.com/list/watchlater', title: '稍后再看', loaded_count: 25, total_count: 25, has_more: false },
          groups: [{ id: 'group:watch', kind: 'watch_later', title: '稍后再看', page: null, items: Array.from({ length: 25 }, (_, i) => ({
            id: `item:watch:${i}`, title: `待看视频 ${i + 1}`, owner_name: '测试用户', cover_url: null, duration_seconds: 60,
            parts: [{ id: `part:watch:${i}`, title: `待看视频 ${i + 1}`, aid: i + 1, bvid: 'BV1xx411c7mD', cid: null, duration_seconds: 60, streams: [], assets: [] }],
          })) }],
        };
        return original(command, args);
      };
    });
    await page.goto('/');
    const input = page.getByRole('textbox', { name: /^Bilibili 链接或 BV \/ AV/ });
    await input.fill('BV17x411w7KC');
    await expect(page.getByRole('checkbox', { name: '返回应用时识别剪贴板链接' })).toHaveCount(0);
    if (device === 'android') {
      await page.getByRole('navigation', { name: '主导航' }).getByRole('button', { name: '我的', exact: true }).click();
      await page.getByRole('button', { name: /下载设置/ }).click();
    } else await page.getByRole('button', { name: /^设置/ }).click();
    await page.getByRole('button', { name: /网络与维护/ }).click();
    await page.getByRole('checkbox', { name: '返回应用时识别剪贴板链接' }).check();
    const prompt = page.getByRole('dialog', { name: '发现剪贴板中的 Bilibili 链接' });
    await expect(prompt).toBeVisible();
    await prompt.getByRole('button', { name: '添加到解析', exact: true }).click();
    await expect(input).toHaveValue('BV17x411w7KC\nhttps://www.bilibili.com/video/BV1xx411c7mD');
    await page.getByRole('button', { name: /^内容库/ }).click();
    await page.getByRole('tab', { name: /^稍后再看/ }).click();
    const cards = page.locator('.library-card');
    await expect(cards).toHaveCount(20);
    await page.getByRole('button', { name: /下一页|Next Page/ }).click();
    await expect(cards).toHaveCount(5);
    await expect(cards.first()).toContainText('待看视频 21');
    await page.getByRole('tab', { name: /^我的收藏/ }).click();
    await expect(page.locator('.library-card').first()).toBeVisible();
    await expect(page.getByRole('button', { name: '刷新内容库' })).toHaveCount(0);
    await expect(page.locator('.library-header')).toHaveCount(0);
    const listCalls = await page.evaluate(() => (window as unknown as { __BDL_TEST_INVOKES__: string[] }).__BDL_TEST_INVOKES__.filter(command => command === 'account_library_list').length);
    await page.getByRole('tab', { name: /^订阅合集/ }).click();
    await page.getByRole('tab', { name: /^我的收藏/ }).click();
    await expect.poll(() => page.evaluate(() => (window as unknown as { __BDL_TEST_INVOKES__: string[] }).__BDL_TEST_INVOKES__.filter(command => command === 'account_library_list').length)).toBe(listCalls + 2);
    await page.evaluate(() => {
      Object.assign(window, { __ISSUE10_CLIP__: 'https://www.bilibili.com/video/BV17x411w7KC' });
      window.dispatchEvent(new Event('focus'));
    });
    await prompt.getByRole('button', { name: '添加到解析', exact: true }).click();
    await expect(input).toHaveValue('BV17x411w7KC\nhttps://www.bilibili.com/video/BV1xx411c7mD\nhttps://www.bilibili.com/video/BV17x411w7KC');
    await page.getByRole('button', { name: /^内容库/ }).click();
    await page.getByRole('tab', { name: /^稍后再看/ }).click();
    await expect(cards).toHaveCount(20);
    await expect(page.getByRole('region', { name: '视频解析', exact: true })).toHaveCount(0);

  });
}

for (const device of ['desktop', 'android'] as const) {
  for (const theme of ['light', 'dark'] as const) {
    test(`issue10 collection tree on ${device} ${theme}`, async ({ page }) => {
      await page.setViewportSize(device === 'desktop' ? { width: 1280, height: 850 } : { width: 390, height: 844 });
      if (device === 'android') await page.addInitScript(() => Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }));
      await installTauriMock(page, theme, true, true);
      await page.addInitScript(() => {
        const target = window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> } };
        const original = target.__TAURI_INTERNALS__.invoke;
        let input = '';
        const large = new URLSearchParams(location.search).has('treeLarge');
        const tree = () => ({
          source: { id: 'collection:tree', kind: 'collection', input, title: '分组测试合集', loaded_count: large ? 300 : 25, total_count: large ? 300 : 25, has_more: false },
          groups: ['5A', '41th', '42th'].map((title, g) => ({ id: `group:${g}`, kind: 'collection_section', title, page: { page_number: 1, page_size: 20, total_count: 25, loaded_count: 25, has_more: false },
            items: Array.from({ length: large ? 100 : g === 2 ? 5 : 10 }, (_, offset) => {
              const i = g * (large ? 100 : 10) + offset;
              return { id: `item:tree:${i}`, title: i === 1 ? '长标题视频：用于验证多端截断与键盘焦点仍然可以正常操作的内容' : `分组视频 ${i + 1}`, owner_name: '测试用户', cover_url: null, duration_seconds: 60,
                parts: Array.from({ length: i === 0 ? 2 : 1 }, (_, part) => ({ id: `part:tree:${i}:${part}`, title: `分 P ${part + 1}`, aid: i + 1, bvid: 'BV1xx411c7mD', cid: i * 10 + part + 1, duration_seconds: 60, streams: [], assets: [] })),
              };
            }),
          })),
        });
        target.__TAURI_INTERNALS__.invoke = async (command, args) => {
          if (command === 'parse_create_source') { input = (args as { request: { input: string } }).request.input; return tree(); }
          if (command === 'parse_load_page') return { tree: tree(), item_ids: Array.from({ length: 5 }, (_, i) => `item:tree:${20 + i}`) };
          return original(command, args);
        };
      });
      await page.goto('/');
      await page.getByRole('textbox', { name: /^Bilibili 链接或 BV \/ AV/ }).fill('BV1kRXLBLEWR');
      await page.getByRole('button', { name: '开始解析', exact: true }).click();
      const tree = page.getByRole('region', { name: '合集分组', exact: true });
      await expect(tree).toBeVisible();
      await expect(tree.getByRole('checkbox', { name: '选择 42th', exact: true })).toBeVisible();
      await tree.getByRole('checkbox', { name: '选择 5A', exact: true }).check();
      await tree.getByRole('button', { name: '展开 5A', exact: true }).click();
      await expect(tree.locator('.tree-scope')).toHaveCount(0);
      const category = tree.locator('.tree-category').filter({ has: page.getByText('5A', { exact: true }) });
      await expect(category).toContainText('10 个视频');
      const mediaRow = tree.locator('.media-list-row').first();
      await expect(mediaRow.locator('.media-play')).toBeVisible();
      await expect(mediaRow.getByLabel('时长')).toHaveText('1:00');
      await expect(page.getByRole('button', { name: '下载所选 (11)', exact: true })).toBeEnabled();
      await tree.getByRole('button', { name: '展开 分组视频 1', exact: true }).click();
      const videoNode = tree.getByRole('treeitem').filter({ has: page.getByText('分组视频 1', { exact: true }) });
      await videoNode.focus();
      await page.keyboard.press('Space');
      await expect(page.getByRole('button', { name: '下载所选 (9)', exact: true })).toBeEnabled();
      await page.keyboard.press('Enter');
      await expect(page.getByRole('button', { name: '下载所选 (11)', exact: true })).toBeEnabled();

      await tree.getByText('P1 · 分 P 1', { exact: true }).click();
      await expect(tree.getByRole('checkbox', { name: '选择 5A', exact: true })).toHaveAttribute('data-state', 'indeterminate');
      await expect(page.getByRole('button', { name: '下载所选 (10)', exact: true })).toBeEnabled();
      await tree.getByRole('button', { name: '展开 41th', exact: true }).focus();
      await page.keyboard.press('Enter');
      await expect(tree.getByRole('button', { name: '收起 41th', exact: true })).toBeVisible();
      await tree.getByRole('button', { name: '播放 分组视频 3', exact: true }).click();
      await expect(page.getByRole('button', { name: '下载所选 (10)', exact: true })).toBeEnabled();
      await tree.locator('.collection-tree-list').evaluate((element) => { element.scrollTop = 0; });
      if (device === 'android') {
        await page.setViewportSize({ width: 1024, height: 600 });
        await expect(tree.getByRole('checkbox', { name: '选择 5A', exact: true })).toHaveAttribute('data-state', 'indeterminate');
      }
      await page.getByRole('button', { name: /^内容库/ }).click();
      await page.getByRole('tab', { name: /^订阅合集/ }).click();
      await page.locator('.library-card').first().click();
      await expect(tree).toBeVisible();
      await tree.getByRole('checkbox', { name: '选择 5A', exact: true }).check();
      await page.getByRole('button', { name: /下一页|Next Page/ }).click();
      await expect(tree.getByRole('checkbox', { name: '选择 5A', exact: true })).toHaveCount(0);
      await tree.getByRole('checkbox', { name: '选择 42th', exact: true }).check();
      await expect(page.getByRole('button', { name: '下载所选 (16)', exact: true })).toBeEnabled();
      await page.getByRole('button', { name: '下载所选 (16)', exact: true }).click();
      await expect(page.getByRole('dialog', { name: '下载设置', exact: true })).toBeVisible();
      if (device === 'desktop' && theme === 'light') {
        await page.goto('/?treeLarge=1');
        await page.getByRole('textbox', { name: /^Bilibili 链接或 BV \/ AV/ }).fill('BV1kRXLBLEWR');
        await page.getByRole('button', { name: '开始解析', exact: true }).click();
        await tree.getByRole('checkbox', { name: '选择 5A', exact: true }).check();
        await tree.getByRole('button', { name: '展开 5A', exact: true }).click();
        await expect(page.getByRole('button', { name: '下载所选 (101)', exact: true })).toBeEnabled();
        expect(await tree.getByRole('treeitem').count()).toBeLessThan(50);
        await tree.locator('.collection-tree-list').evaluate((element) => { element.scrollTop = element.scrollHeight; });
        await tree.getByRole('checkbox', { name: '选择 分组视频 100', exact: true }).uncheck();
        await expect(page.getByRole('button', { name: '下载所选 (100)', exact: true })).toBeEnabled();
      }

    });
  }
}

for (const device of ['desktop', 'android'] as const) {
  for (const theme of ['light', 'dark'] as const) {
    test(`watch later browse details ${device} ${theme}`, async ({ page }) => {
      await page.setViewportSize(device === 'desktop' ? { width: 1280, height: 850 } : { width: 390, height: 844 });
      if (device === 'android') await page.addInitScript(() => Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }));
      await installTauriMock(page, theme, true, true);
      await page.addInitScript(() => {
        const target = window as any;
        const original = target.__TAURI_INTERNALS__.invoke;
        target.__DETAIL_REQUESTS__ = [];
        const part = (i: number) => ({ id: `part:${i}`, title: `分 P ${i + 1}`, aid: i + 1, bvid: 'BV1xx411c7mD', cid: i + 100, duration_seconds: 60, streams: [], assets: [] });
        const item = (i: number) => ({ id: `item:${i}`, title: `待看视频 ${i + 1}`, owner_name: '测试用户', cover_url: null, duration_seconds: 60, parts: [{ ...part(i), bvid: i === 0 ? 'BV1xx411c7mD' : i === 1 ? 'BV17x411w7KC' : 'BV1kRXLBLEWR', cid: null }] });
        target.__TAURI_INTERNALS__.invoke = async (command: string, args: any) => {
          if (command === 'parse_create_source') {
            target.__DETAIL_REQUESTS__.push(args.request);
            if (args.request.input.includes('watchlater')) return {
              source: { id: 'watch_later:browse', kind: 'watch_later', input: args.request.input, title: '稍后再看', loaded_count: 25, total_count: 25, has_more: false },
              groups: [{ id: 'group:watch', kind: 'watch_later', title: '稍后再看', page: null, items: Array.from({ length: 25 }, (_, i) => item(i)) }],
            };
            if (args.request.input.includes('BV1xx411c7mD')) return {
              source: { id: 'collection:detail', kind: 'collection', input: 'https://space.bilibili.com/1/lists/2?type=season', title: '视频所属合集：20分钟精选互联网十年最爆笑猫咪合集，笑到停不下来 | Daily Dose of Internet 超长标题测试', loaded_count: 20, total_count: 61, has_more: true },
              groups: [{ id: 'group:collection', kind: 'collection', title: '视频所属合集：20分钟精选互联网十年最爆笑猫咪合集，笑到停不下来 | Daily Dose of Internet 超长标题测试', page: { page: 1, page_size: 20, total_count: 61, has_more: true }, items: Array.from({ length: 20 }, (_, i) => item(i)) }],
            };
            const count = args.request.input.includes('BV17x411w7KC') ? 25 : 1;
            return {
              source: { id: 'video:detail', kind: 'video', input: count === 25 ? 'BV17x411w7KC' : 'BV1kRXLBLEWR', title: (count === 25 ? '多分 P 视频' : '普通视频') + '：20分钟精选互联网十年最爆笑猫咪合集，笑到停不下来 | Daily Dose of Internet 超长标题测试', loaded_count: 1, total_count: 1, has_more: false },
              groups: [{ id: 'group:video', kind: 'video', title: '视频', page: null, items: [{ ...item(1), parts: Array.from({ length: count }, (_, i) => part(i)) }] }],
            };
          }
          return original(command, args);
        };
      });
      await page.goto('/');
      await page.getByRole('button', { name: /^内容库/ }).click();
      await page.getByRole('tab', { name: /^稍后再看/ }).click();
      const cards = page.locator('.library-card');
      await expect(cards).toHaveCount(20);
      expect(await page.evaluate(() => (window as any).__DETAIL_REQUESTS__.length)).toBe(1);
      await expect(page.getByRole('region', { name: '视频解析', exact: true })).toHaveCount(0);
      await cards.first().focus();
      await page.keyboard.press('Enter');
      const rows = page.locator(device === 'desktop' ? '.media-list-row' : '.mobile-result');
      await expect(rows.first()).toBeVisible();
      await expect(page.locator('h2').filter({ hasText: '视频所属合集' })).toBeVisible();
      if (device === 'desktop') {
        await expect(page.getByRole('button', { name: '解析', exact: true })).toBeVisible();
        await expect(page.getByRole('combobox', { name: '每批解析数量' })).toBeVisible();
        await page.getByRole('button', { name: '更多解析方式', exact: true }).click();
        await expect(page.getByRole('menuitem', { name: '解析全部', exact: true })).toBeVisible();
        await page.keyboard.press('Escape');
      } else {
        await page.getByRole('button', { name: '内容操作', exact: true }).click();
        await expect(page.getByRole('button', { name: '加载全部内容', exact: true })).toBeVisible();
        await page.keyboard.press('Escape');
      }
      await page.getByRole('button', { name: '返回内容集合', exact: true }).click();
      await expect(cards).toHaveCount(20);
      await cards.nth(1).click();
      await expect(rows.first()).toContainText('分 P 1');
      await expect(page.locator('h2').filter({ hasText: '多分 P 视频' })).toBeVisible();
      await page.getByRole('button', { name: /下一页|Next Page/ }).click();
      await expect(rows).toHaveCount(5);
      await expect(rows.first()).toContainText('分 P 21');
      await rows.first().click({ position: { x: 2, y: 2 } });
      await expect(page.getByRole('button', { name: '下载所选 (1)', exact: true })).toBeEnabled();
      await page.getByRole('button', { name: '下载所选 (1)', exact: true }).click();
      await expect(page.getByRole('dialog', { name: '下载设置', exact: true })).toHaveCount(1);
      await page.keyboard.press('Escape');
      await page.getByRole('button', { name: '返回内容集合', exact: true }).click();
      await cards.nth(2).click();
      await expect(rows).toHaveCount(1);
      await expect(page.locator('h2').filter({ hasText: '普通视频' })).toBeVisible();
      for (const width of device === 'desktop' ? [1280, 1000, 800, 375] : [390, 1024]) {
        await page.setViewportSize({ width, height: 850 });
        const title = page.locator(device === 'desktop' ? '.library-folder-header h2' : '.mobile-detail-copy h2');
        await expect(title).toHaveAttribute('title', /超长标题测试/);
        if (device === 'desktop' && width > 700) {
          await title.hover();
        }
      }
      expect(await page.evaluate(() => (window as any).__DETAIL_REQUESTS__.slice(1).every((r: any) => r.expand_video_collection))).toBe(true);
    });
  }
}

for (const device of ['desktop', 'android'] as const) {
  for (const theme of ['light', 'dark'] as const) {
    test(`library refresh skeleton ${device} ${theme}`, async ({ page }) => {
      await page.setViewportSize(device === 'desktop' ? { width: 1280, height: 850 } : { width: 390, height: 844 });
      if (device === 'android') await page.addInitScript(() => Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' }));
      await installTauriMock(page, theme, true, true);
      await page.addInitScript(() => {
        const target = window as any;
        const original = target.__TAURI_INTERNALS__.invoke;
        target.__TAURI_INTERNALS__.invoke = async (command: string, args: any) => {
          if (command === 'account_library_list' && target.__HOLD_LIBRARY__) {
            await new Promise<void>(resolve => { target.__RELEASE_LIBRARY__ = resolve; });
          }
          return original(command, args);
        };
      });
      await page.goto('/');
      await page.getByRole('button', { name: /^内容库/ }).click();
      await expect(page.locator('.library-card[role="button"]').first()).toBeVisible();
      await page.getByRole('tab', { name: /^订阅合集/ }).click();
      await expect(page.locator('.library-card[role="button"]').first()).toBeVisible();
      for (const category of ['我的收藏', '订阅合集']) {
        await page.evaluate(() => { (window as any).__HOLD_LIBRARY__ = true; (window as any).__RELEASE_LIBRARY__ = null; });
        await page.getByRole('tab', { name: new RegExp(`^${category}`) }).focus();
        await page.keyboard.press('Enter');
        await expect(page.getByRole('status', { name: '正在加载内容', exact: true })).toBeVisible();
        await expect(page.locator('.library-card.skeleton')).toHaveCount(6);
        await expect(page.locator('.library-card[role="button"]')).toHaveCount(0);
        await expect(page.locator('.library-sync')).toHaveCount(0);
        await expect.poll(() => page.evaluate(() => typeof (window as any).__RELEASE_LIBRARY__)).toBe('function');
        await page.evaluate(() => { (window as any).__HOLD_LIBRARY__ = false; (window as any).__RELEASE_LIBRARY__(); });
        await expect(page.locator('.library-card[role="button"]').first()).toBeVisible();
        await expect(page.locator('.skeleton')).toHaveCount(0);
      }
    });
  }
}


for (const device of ['desktop', 'phone'] as const) {
  for (const theme of ['light', 'dark'] as const) {
    test(`parse failure diagnostics ${device} ${theme}`, async ({ page }) => {
      await page.setViewportSize({ width: device === 'phone' ? 375 : 1280, height: 800 });
      await installTauriMock(page, theme);
      await page.addInitScript(({ phone }) => {
        if (phone) Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' });
        const target = window as unknown as {
          __TAURI_INTERNALS__: { invoke: (command: string, args?: unknown) => Promise<unknown> };
          copiedDiagnostic?: string;
          failCopy?: boolean;
          feedbackUrl?: string;
        };
        Object.defineProperty(navigator, 'clipboard', { value: { writeText: async (text: string) => {
          if (target.failCopy) throw new Error('clipboard denied');
          target.copiedDiagnostic = text;
        } } });
        const invoke = target.__TAURI_INTERNALS__.invoke;
        target.__TAURI_INTERNALS__.invoke = async (command, args) => {
          if (command === 'parse_create_source') throw {
            code: 'bilibili_api_error', message: 'bpi error: failed to decode response',
            detail: 'HTTP 412; response field missing; SESSDATA=secret-cookie; ' + 'long_reason_'.repeat(100),
          };
          if (command === 'open_external_url') { target.feedbackUrl = (args as { url: string }).url; return; }
          return invoke(command, args);
        };
      }, { phone: device === 'phone' });
      await page.goto('/');
      await page.getByLabel('Bilibili 链接或 BV / AV').fill('BV1xx411c7mD');
      await page.getByRole('button', { name: '开始解析' }).click();
      const error = page.locator('.error-details');
      await expect(error).toBeVisible();
      const summary = error.locator('summary');
      await summary.focus();
      await page.keyboard.press('Enter');
      await expect(error.locator('pre')).toBeVisible();
      await expect(error.locator('pre')).toContainText('HTTP 412');
      await expect(error.locator('pre')).not.toContainText('secret-cookie');
      await error.getByRole('button', { name: '复制诊断信息' }).click();
      await expect(error.getByRole('button', { name: '已复制' })).toBeVisible();
      const copied = await page.evaluate(() => (window as unknown as { copiedDiagnostic: string }).copiedDiagnostic);
      expect(copied).toContain('bilibili_api_error');
      expect(copied).toContain('BV1xx411c7mD');
      expect(copied).toContain('当前环境');
      expect(copied).not.toContain('secret-cookie');
      await error.getByRole('button', { name: '反馈问题' }).click();
      expect(await page.evaluate(() => (window as unknown as { feedbackUrl: string }).feedbackUrl)).toBe('https://github.com/Yuelioi/bdl/issues/new/choose');
      await page.evaluate(() => { (window as unknown as { failCopy: boolean }).failCopy = true; });
      await error.getByRole('button', { name: '已复制' }).click();
      await expect(error.getByText('复制失败，请展开错误详情后手动复制。')).toBeVisible();
      await summary.click();
      await expect(error.locator('pre')).not.toBeVisible();
    });
  }
}

for (const device of ['desktop', 'phone'] as const) {
  test(`library failure chain ${device}`, async ({ page }) => {
    await page.setViewportSize({ width: device === 'phone' ? 375 : 1280, height: 800 });
    await installTauriMock(page, 'light', true, true);
    await page.addInitScript(({ phone }) => {
      if (phone) Object.defineProperty(navigator, 'userAgent', { value: 'Mozilla/5.0 (Linux; Android 12) Mobile' });
      const target = window as unknown as { __TAURI_INTERNALS__: { invoke: (cmd: string, args?: unknown) => Promise<unknown> } };
      const invoke = target.__TAURI_INTERNALS__.invoke;
      target.__TAURI_INTERNALS__.invoke = async (cmd, args) => {
        if (cmd === 'account_library_list' || cmd === 'parse_create_source') throw { code: 'bilibili_api_error', message: '加载失败', detail: 'HTTP 429; token=secret-value' };
        return invoke(cmd, args);
      };
    }, { phone: device === 'phone' });
    await page.goto('/');
    await page.getByRole('button', { name: device === 'phone' ? '内容库' : '内容库 收藏与订阅', exact: true }).click();
    for (const category of ['我的收藏', '订阅合集', '稍后再看']) {
      await page.getByRole('tab', { name: new RegExp(category) }).click();
      const details = page.locator('.error-details');
      await expect(details.getByRole('button', { name: '复制诊断信息' })).toBeVisible();
      await details.locator('summary').click();
      await expect(details.locator('pre')).toBeVisible();
      await expect(details.locator('pre')).toContainText('HTTP 429');
      await expect(details.locator('pre')).not.toContainText('secret-value');
    }
  });
}

test('pagination and background keep real causes from transport to controls', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.addInitScript(() => {
    const target = window as unknown as { __TAURI_INTERNALS__: { invoke: (cmd: string, args?: unknown) => Promise<unknown> }; attempts?: number };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.attempts = 0;
    target.__TAURI_INTERNALS__.invoke = async (cmd, args) => {
      if (cmd === 'selection_create_tasks') target.attempts!++;
      if (cmd === 'parse_load_more' || cmd === 'selection_create_tasks') throw { code: 'bilibili_api_error', message: '操作失败', detail: 'HTTP 429: retry later; SESSDATA=hidden-cookie' };
      return invoke(cmd, args);
    };
  });
  await page.goto('/');
  await page.getByLabel('链接或 BV / AV').fill('favorite:fixture');
  await page.getByRole('button', { name: '开始解析' }).click();
  await page.getByRole('button', { name: '解析', exact: true }).click();
  await expect(page.locator('.error-details')).toBeVisible();
  await page.locator('.error-details summary').click();
  await expect(page.locator('.error-details pre')).toContainText('HTTP 429');
  await page.getByRole('button', { name: '更多解析方式' }).click();
  await page.getByRole('menuitem', { name: '后台解析全部并下载' }).click();
  const status = page.getByRole('region', { name: '后台解析下载' });
  await expect(status).toContainText('后台解析下载失败');
  await status.locator('summary').click();
  await expect(status.locator('pre')).toContainText('HTTP 429');
  await expect(status.locator('pre')).not.toContainText('hidden-cookie');
  expect(await page.evaluate(() => (window as unknown as { attempts: number }).attempts)).toBe(1);
});

test('recent failure report survives page changes and can be cleared', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.addInitScript(() => {
    const target = window as unknown as { exportedErrors?: string; __TAURI_INTERNALS__: { invoke: (cmd: string, args?: unknown) => Promise<unknown> } };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (cmd, args) => {
      if (cmd === 'diagnostics_export') { target.exportedErrors = (args as { recentErrors: string }).recentErrors; return { path: 'fixture-diagnostics.json' }; }
      if (cmd === 'parse_create_source') throw { code: 'bilibili_api_error', message: '失败', detail: 'API diagnostic sentinel; token=do-not-copy' };
      return invoke(cmd, args);
    };
  });
  await page.goto('/');
  await page.getByLabel('链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();
  await expect(page.locator('.error-details')).toBeVisible();
  await page.getByRole('button', { name: '设置 偏好与维护' }).click();
  await page.getByRole('button', { name: /网络与维护/ }).click();
  await page.getByText('最近的操作错误', { exact: true }).click();
  await page.locator('.error-details summary').click();
  await expect(page.locator('.error-details pre')).toContainText('API diagnostic sentinel');
  await expect(page.locator('.error-details pre')).not.toContainText('do-not-copy');
  await page.getByRole('button', { name: '导出诊断', exact: true }).click();
  const exported = await page.evaluate(() => (window as unknown as { exportedErrors: string }).exportedErrors);
  expect(exported).toContain('API diagnostic sentinel');
  expect(exported).not.toContain('do-not-copy');
  await page.getByRole('button', { name: '清空错误记录' }).click();
  await expect(page.getByText('最近的操作错误', { exact: true })).toHaveCount(0);
});

test('size estimation failure is inspectable without disabling download', async ({ page }) => {
  await installTauriMock(page, 'light');
  await page.addInitScript(() => {
    const target = window as unknown as { __TAURI_INTERNALS__: { invoke: (cmd: string, args?: unknown) => Promise<unknown> } };
    const invoke = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (cmd, args) => {
      if (cmd === 'selection_estimate_size') throw { code: 'bilibili_api_error', message: '解析失败', detail: 'estimate-cause-sentinel' };
      return invoke(cmd, args);
    };
  });
  await page.goto('/');
  await page.getByLabel('链接或 BV / AV').fill('BV1xx411c7mD');
  await page.getByRole('button', { name: '开始解析' }).click();
  await page.getByRole('button', { name: '选择 测试视频' }).click();
  await page.getByRole('button', { name: '下载所选 (1)' }).click();
  const dialog = page.getByRole('dialog', { name: '下载设置' });
  await expect(dialog.getByRole('button', { name: '复制诊断信息' })).toBeVisible();
  await dialog.locator('.error-details summary').click();
  await expect(dialog.locator('pre')).toContainText('estimate-cause-sentinel');
  await expect(dialog.getByRole('button', { name: '开始下载' })).toBeEnabled();
});
