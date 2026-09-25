import { describe, expect, it } from 'vitest';
import { coverUrl } from './coverUrl';

describe('coverUrl', () => {
  it('uses HTTPS for legacy and protocol-relative Bilibili covers', () => {
    expect(coverUrl('http://i0.hdslb.com/bfs/archive/cover.jpg')).toBe('https://i0.hdslb.com/bfs/archive/cover.jpg');
    expect(coverUrl('//i1.hdslb.com/bfs/archive/cover.jpg')).toBe('https://i1.hdslb.com/bfs/archive/cover.jpg');
  });
  it('preserves HTTPS query strings and missing covers', () => {
    expect(coverUrl('https://i0.hdslb.com/cover.jpg?size=80')).toBe('https://i0.hdslb.com/cover.jpg?size=80');
    expect(coverUrl(null)).toBeNull();
    expect(coverUrl(' ')).toBeNull();
  });
});
