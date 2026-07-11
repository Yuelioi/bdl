import { describe, expect, it, vi } from 'vitest';

import {
  useTransferTaskActions,
  type TransferTaskAction,
  type TransferTaskActionQueue,
} from './useTransferTaskActions';

const queue = (): TransferTaskActionQueue => ({
  pause: vi.fn(),
  resume: vi.fn(),
  unschedule: vi.fn(),
  retry: vi.fn(),
  refreshUrlsAndRetry: vi.fn(),
  cancel: vi.fn(),
  remove: vi.fn(),
  openFile: vi.fn(),
  openDir: vi.fn(),
  copySource: vi.fn(),
});

describe('transfer task actions', () => {
  it.each<readonly [TransferTaskAction, keyof TransferTaskActionQueue]>([
    ['pause', 'pause'],
    ['resume', 'resume'],
    ['unschedule', 'unschedule'],
    ['retry', 'retry'],
    ['refresh_retry', 'refreshUrlsAndRetry'],
    ['cancel', 'cancel'],
    ['remove', 'remove'],
    ['open_file', 'openFile'],
    ['open_dir', 'openDir'],
    ['copy_source', 'copySource'],
  ])('dispatches %s to queue.%s', (action, method) => {
    const store = queue();
    const taskActions = useTransferTaskActions(store, {
      openScheduleDialog: vi.fn(),
      openSpeedLimitDialog: vi.fn(),
    });

    taskActions.runTaskAction('task:one', action);

    expect(store[method]).toHaveBeenCalledWith('task:one');
  });

  it('routes local configuration actions to their dialogs', () => {
    const dialogs = {
      openScheduleDialog: vi.fn(),
      openSpeedLimitDialog: vi.fn(),
    };
    const taskActions = useTransferTaskActions(queue(), dialogs);

    taskActions.runTaskAction('task:schedule', 'schedule');
    taskActions.runTaskAction('task:speed', 'speed_limit');

    expect(dialogs.openScheduleDialog).toHaveBeenCalledWith('task:schedule');
    expect(dialogs.openSpeedLimitDialog).toHaveBeenCalledWith('task:speed');
  });
});
