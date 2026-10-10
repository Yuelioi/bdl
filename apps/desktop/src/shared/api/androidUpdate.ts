export interface AndroidUpdate {
  version: string;
  notes: string | null;
  downloadUrl: string;
}

const versionParts = (version: string): number[] => {
  const match = /^v?(\d+)\.(\d+)\.(\d+)$/.exec(version);
  if (!match) throw new Error('更新版本号无效。');
  return match.slice(1).map(Number);
};

export async function checkAndroidUpdate(currentVersion: string): Promise<AndroidUpdate | null> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 15_000);
  try {
    const response = await fetch('https://api.github.com/repos/Yuelioi/bdl/releases/latest', {
      headers: { Accept: 'application/vnd.github+json' },
      signal: controller.signal,
    });
    if (!response.ok) throw new Error(`检测更新失败（HTTP ${response.status}），请稍后重试。`);
    const release = (await response.json()) as {
      tag_name: string;
      draft: boolean;
      prerelease: boolean;
      body?: string | null;
      assets: Array<{ name: string; size: number; browser_download_url: string }>;
    };
    if (release.draft || release.prerelease) throw new Error('尚无可用的正式更新。');
    const latest = versionParts(release.tag_name);
    const current = versionParts(currentVersion);
    const differing = latest.findIndex((part, index) => part !== current[index]);
    if (differing === -1 || latest[differing] < current[differing]) return null;
    const version = latest.join('.');
    const name = `BDL-v${version}-android-arm64.apk`;
    const url = `https://github.com/Yuelioi/bdl/releases/download/${encodeURIComponent(release.tag_name)}/${name}`;
    const asset = Array.isArray(release.assets)
      ? release.assets.find((item) => item.name === name && item.size > 0 && item.browser_download_url === url)
      : undefined;
    if (!asset) throw new Error('新版本的 Android 安装包尚未就绪，请稍后重试。');
    return { version, notes: release.body ?? null, downloadUrl: asset.browser_download_url };
  } catch (error) {
    if (controller.signal.aborted) throw new Error('检测更新超时，请稍后重试。');
    throw error;
  } finally {
    clearTimeout(timer);
  }
}
