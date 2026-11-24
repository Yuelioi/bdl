import { describe, expect, it } from 'vitest';
import type { NormalizedSourceTree } from '../../api/dto';
import { collectionTree, hasCollectionSections, treeSelection } from './collectionTree';

const source: NormalizedSourceTree = {
  source: {
    id: 'collection:1',
    kind: 'collection',
    title: '合集',
    input: '',
    loaded_count: 2,
    total_count: 4,
    has_more: true,
  },
  groups: [1, 2].map((id) => ({
    id: `g${id}`,
    kind: 'collection_section',
    title: `分组 ${id}`,
    page: null,
    items: [
      {
        id: `v${id}`,
        title: `视频 ${id}`,
        owner_name: 'UP',
        cover_url: null,
        duration_seconds: 0,
        parts: [1, 2].map((part) => ({
          id: `p${id}-${part}`,
          title: `分 P ${part}`,
          aid: id,
          bvid: null,
          cid: part,
          duration_seconds: 0,
          streams: [],
          assets: [],
        })),
      },
    ],
  })),
};
describe('collection tree selection scope', () => {
  it('selects each collection video as a whole within its section', () => {
    expect(hasCollectionSections(source)).toBe(true);
    const root = collectionTree(source)[0];
    expect(root.children?.map((group) => group.label)).toEqual(['分组 1', '分组 2']);
    expect(root.children?.[0].children?.[0]).toMatchObject({ id: 'v1', partIds: ['p1-1'] });
    expect(root.children?.[0].children?.[0].children).toBeUndefined();
    expect(treeSelection(root.partIds, new Set(['p1-1']))).toBe('indeterminate');
    expect(treeSelection(root.partIds, new Set(root.partIds))).toBe(true);
    expect(treeSelection([], new Set())).toBe(false);
  });
  it('limits group selection to the visible page and search result without including hidden items', () => {
    const page = collectionTree(source, ['v2'])[0];
    expect(page.children).toHaveLength(1);
    expect(page.partIds).toEqual(['p2-1']);
    expect(collectionTree(source, undefined, '分组 1')[0].partIds).toEqual(['p1-1']);
    expect(collectionTree(source, ['v1'], '视频 2')[0].partIds).toEqual([]);
  });
});
