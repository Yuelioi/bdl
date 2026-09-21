export const isMacPlatform = (
  platform = globalThis.navigator?.platform ?? '',
  userAgent = globalThis.navigator?.userAgent ?? '',
): boolean => /^(Mac|iPhone|iPad)/.test(platform) || /Macintosh/.test(userAgent)

export const isAndroidPlatform = (
  userAgent = globalThis.navigator?.userAgent ?? '',
): boolean => /Android/i.test(userAgent)

export const isIosPlatform = (
  platform = globalThis.navigator?.platform ?? '',
  userAgent = globalThis.navigator?.userAgent ?? '',
): boolean =>
  /^(iPhone|iPad|iPod)/.test(platform) ||
  /(?:iPhone|iPad|iPod)/.test(userAgent) ||
  (/Macintosh/.test(userAgent) && (globalThis.navigator?.maxTouchPoints ?? 0) > 1)

export const isMobilePlatform = (
  platform = globalThis.navigator?.platform ?? '',
  userAgent = globalThis.navigator?.userAgent ?? '',
): boolean => isAndroidPlatform(userAgent) || isIosPlatform(platform, userAgent)
