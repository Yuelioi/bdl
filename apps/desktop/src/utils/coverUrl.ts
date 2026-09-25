/** Bilibili image URLs may use HTTP or omit the scheme; Android requires HTTPS. */
export function coverUrl(value: string | null | undefined): string | null {
  const url = value?.trim();
  if (!url) return null;
  if (url.startsWith('//')) return `https:${url}`;
  if (url.startsWith('http://')) return `https://${url.slice(7)}`;
  return url;
}
