import { describe, expect, it } from 'vitest';

import type { DownloadTask } from '../../api/dto';
import { isCancellable, isPausable, isRetryable, matchesTransferSearch, sortTransferTasks } from './transferQueries';

const task = (id: string, title: string, status: DownloadTask['status'] = 'waiting'): DownloadTask =>
  ({ id, title, status, source_id: `source:${id}`, output_path: `downloads/${id}.mp4` }) as DownloadTask;

const metrics = {
  progress: (value: DownloadTask) => ({ one: 30, two: 80, three: 80 })[value.id] ?? 0,
  speed: (id: string) => ({ one: 4, two: 1, three: 8 })[id] ?? 0,
};

describe('transfer queries', () => {
  const tasks = [task('one', '视频 10'), task('two', '视频 2'), task('three', '视频 2', 'failed')];

  it('keeps stable ordering when progress values are equal', () => {
    expect(sortTransferTasks(tasks, 'progress_desc', metrics).map((value) => value.id)).toEqual([
      'two',
      'three',
      'one',
    ]);
  });

  it('puts actionable failures before active and completed tasks', () => {
    const completed = task('done', '完成', 'completed');
    expect(sortTransferTasks([...tasks, completed], 'issue_first', metrics)[0]?.id).toBe('three');
  });

  it('searches title, source and output path case-insensitively', () => {
    expect(matchesTransferSearch(tasks[0]!, 'ONE', 'UP 主')).toBe(true);
  });

  it('defines bulk action eligibility from task status', () => {
    expect([isPausable('downloading'), isCancellable('paused'), isRetryable('completed')]).toEqual([true, true, true]);
  });
});
