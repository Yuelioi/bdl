import { describe, expect, it, vi } from 'vitest';

import type { DownloadTask } from '../../api/dto';
import { toDateTimeLocalValue } from '../../utils/schedule';
import { useTransferDialogs, type TransferDialogQueue } from './useTransferDialogs';

const task = (overrides: Partial<DownloadTask> = {}): DownloadTask =>
  ({ id: 'task:one', scheduled_at: null, speed_limit_bytes_per_second: null, ...overrides }) as DownloadTask;

const queue = (tasks = [task()]) =>
  ({ tasks, schedule: vi.fn(), setSpeedLimit: vi.fn() }) satisfies TransferDialogQueue;

describe('transfer dialogs', () => {
  it('opens scheduling with deterministic minimum and default time', () => {
    const now = Date.UTC(2026, 6, 11, 2, 0);
    const dialogs = useTransferDialogs(queue(), () => now);

    dialogs.openScheduleDialog('task:one');

    expect(dialogs.scheduleMin.value).toBe(toDateTimeLocalValue(new Date(now + 60_000)));
    expect(dialogs.scheduleLocal.value).toBe(toDateTimeLocalValue(new Date(now + 300_000)));
  });

  it('keeps an invalid schedule dialog open without calling the queue', async () => {
    const store = queue();
    const dialogs = useTransferDialogs(store, () => Date.UTC(2026, 6, 11, 2, 0));
    dialogs.openScheduleDialog('task:one');
    dialogs.scheduleLocal.value = '';

    expect(await dialogs.submitSchedule()).toBe(false);
    expect(store.schedule).not.toHaveBeenCalled();
    expect(dialogs.scheduleDialogOpen.value).toBe(true);
  });

  it('loads and submits an existing speed limit', async () => {
    const store = queue([task({ speed_limit_bytes_per_second: 2_621_440 })]);
    store.setSpeedLimit.mockResolvedValue(true);
    const dialogs = useTransferDialogs(store);
    dialogs.openSpeedLimitDialog('task:one');

    expect(dialogs.speedLimitMib.value).toBe('2.5');
    expect(await dialogs.submitSpeedLimit()).toBe(true);
    expect(store.setSpeedLimit).toHaveBeenCalledWith('task:one', 2_621_440);
  });
});
