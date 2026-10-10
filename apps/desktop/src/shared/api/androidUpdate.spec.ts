import { afterEach, describe, expect, it, vi } from 'vitest';
import { checkAndroidUpdate } from './androidUpdate';

const release = () => ({
  tag_name: 'v0.8.10',
  draft: false,
  prerelease: false,
  body: '修复手机端',
  assets: [
    {
      name: 'BDL-v0.8.10-android-arm64.apk',
      size: 100,
      browser_download_url: 'https://github.com/Yuelioi/bdl/releases/download/v0.8.10/BDL-v0.8.10-android-arm64.apk',
    },
  ],
});
const mockRelease = (value = release(), status = 200) => {
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(JSON.stringify(value), { status })));
};
afterEach(() => vi.unstubAllGlobals());

describe('Android release checks', () => {
  it('compares numeric versions and selects the official Android APK', async () => {
    mockRelease();
    expect(await checkAndroidUpdate('0.8.9')).toMatchObject({
      version: '0.8.10',
      notes: '修复手机端',
      downloadUrl: release().assets[0].browser_download_url,
    });
  });
  it.each(['0.8.10', '0.9.0'])('does not offer a downgrade from %s', async (current) => {
    mockRelease();
    expect(await checkAndroidUpdate(current)).toBeNull();
  });
  it('reports missing Android artifacts instead of claiming the installed version is latest', async () => {
    mockRelease({ ...release(), assets: [] });
    await expect(checkAndroidUpdate('0.8.9')).rejects.toThrow('安装包尚未就绪');
  });
  it('rejects a release asset pointing outside the official repository', async () => {
    const value = release();
    value.assets[0].browser_download_url = 'https://example.com/app.apk';
    mockRelease(value);
    await expect(checkAndroidUpdate('0.8.9')).rejects.toThrow('安装包尚未就绪');
  });
  it('reports API failures', async () => {
    mockRelease(release(), 403);
    await expect(checkAndroidUpdate('0.8.9')).rejects.toThrow('HTTP 403');
  });
});
