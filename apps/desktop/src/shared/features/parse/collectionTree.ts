import type { NormalizedSourceTree } from '../../api/dto';
import { selectableParts } from './parseResultTree';

export interface CollectionTreeNode {
  id: string;
  label: string;
  kind: 'source' | 'group' | 'video';
  partIds: string[];
  cover?: string | null;
  meta?: string;
  children?: CollectionTreeNode[];
}

export const hasCollectionSections = (source: NormalizedSourceTree | null): boolean =>
  source?.source.kind === 'collection' && source.groups.some((group) => group.kind === 'collection_section');

export function collectionTree(source: NormalizedSourceTree, itemIds?: string[], query = ''): CollectionTreeNode[] {
  const allowed = itemIds ? new Set(itemIds) : null;
  const keyword = query.trim().toLocaleLowerCase();
  const groups = source.groups.flatMap((group): CollectionTreeNode[] => {
    const groupMatches = group.title.toLocaleLowerCase().includes(keyword);
    const videos = group.items.flatMap((item): CollectionTreeNode[] => {
      if (allowed && !allowed.has(item.id)) return [];
      if (
        keyword &&
        !groupMatches &&
        ![item.title, item.owner_name].some((text) =>
          text?.toLocaleLowerCase().includes(keyword),
        )
      )
        return [];
      return [
        {
          id: item.id,
          label: item.title,
          kind: 'video',
          cover: item.cover_url,
          meta: item.owner_name ?? undefined,
          partIds: selectableParts(item, source.source.kind).map((part) => part.id),
        },
      ];
    });
    if (!videos.length && (allowed || keyword || group.kind !== 'collection_section')) return [];
    return [
      {
        id: group.id,
        label: group.title,
        kind: 'group',
        meta: `${videos.length} 个视频`,
        partIds: videos.flatMap((video) => video.partIds),
        children: videos,
      },
    ];
  });
  return [
    {
      id: source.source.id,
      label: source.source.title,
      kind: 'source',
      partIds: groups.flatMap((group) => group.partIds),
      children: groups,
    },
  ];
}

export function treeSelection(ids: string[], selected: Set<string>): boolean | 'indeterminate' {
  const count = ids.filter((id) => selected.has(id)).length;
  return count === 0 ? false : count === ids.length ? true : 'indeterminate';
}
