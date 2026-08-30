export const isMacPlatform = (
  platform = globalThis.navigator?.platform ?? '',
  userAgent = globalThis.navigator?.userAgent ?? '',
): boolean => /^(Mac|iPhone|iPad)/.test(platform) || /Macintosh/.test(userAgent)
