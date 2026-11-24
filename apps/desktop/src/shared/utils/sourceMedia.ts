import type { DownloadTask, NormalizedItem, NormalizedPart, NormalizedSourceTree } from '../api/dto';
import { selectsWholePosts } from '../features/parse/parseResultTree';

export interface SourceMedia {
  part: NormalizedPart;
  item: NormalizedItem;
  page: number;
  wholePost: boolean;
  cover: string | null;
  secondary: string;
  published: string | null;
  duration: number | null;
  url: string | null;
}

export function sourceMediaByPart(tree: NormalizedSourceTree, fallbackOwner?: string | null): Map<string, SourceMedia> {
  const wholePost = selectsWholePosts(tree.source.kind);
  return new Map(
    tree.groups.flatMap((group) =>
      group.items.flatMap((item) =>
        item.parts.map((part, index) => {
          const page = index + 1;
          const video = part.bvid ? part.bvid : part.aid ? `av${part.aid}` : null;
          const episode = part.id.match(/^part:(bangumi|cheese):.*:(\d+):\d+$/);
          const url = episode
            ? `https://www.bilibili.com/${episode[1]}/play/ep${episode[2]}`
            : video
              ? `https://www.bilibili.com/video/${video}${!wholePost && item.parts.length > 1 ? `?p=${page}` : ''}`
              : null;
          return [
            part.id,
            {
              item,
              part,
              page,
              wholePost,
              cover: item.cover_url,
              secondary: item.owner_name?.trim() || fallbackOwner?.trim() || tree.source.title,
              published: item.publish_date || null,
              duration: wholePost ? item.duration_seconds : part.duration_seconds ?? item.duration_seconds,
              url,
            },
          ] as const;
        }),
      ),
    ),
  );
}

export function completedTaskForMedia(media: SourceMedia, tasks: DownloadTask[]): DownloadTask | undefined {
  return tasks.find(
    (task) =>
      task.status === 'completed' &&
      ((matchesCompletedPart(media, task) &&
        (!media.wholePost || task.media_selection.post || (task.refresh_intent?.page_number ?? 1) === 1)) ||
        task.media_selection.post?.finished.some((part) => matchesCompletedPart(media, part))),
  );
}

function matchesCompletedPart(media: SourceMedia, part: DownloadTask): boolean {
  if (!part.output_path || !part.refresh_intent) return false;
  const intent = part.refresh_intent;
  if (!media.wholePost && media.part.cid != null && intent.cid !== media.part.cid) return false;
  if (intent.input.kind === 'video_bvid') {
    if (intent.input.bvid !== media.part.bvid) return false;
  } else if (intent.input.kind === 'video_aid') {
    if (intent.input.aid !== media.part.aid) return false;
  } else {
    const kind = intent.input.kind === 'bangumi_episode' ? 'bangumi' : 'cheese';
    if (media.url !== `https://www.bilibili.com/${kind}/play/ep${intent.input.ep_id}`) return false;
  }
  return media.wholePost || media.part.cid != null || (intent.page_number ?? 1) === media.page;
}
