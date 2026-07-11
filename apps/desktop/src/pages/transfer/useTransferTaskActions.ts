import type { TaskActionKind } from '../../stores/transferView';

export type TransferTaskAction = Exclude<TaskActionKind, 'none'>;

export interface TransferTaskActionQueue {
  pause(taskId: string): Promise<unknown>;
  resume(taskId: string): Promise<unknown>;
  unschedule(taskId: string): Promise<unknown>;
  retry(taskId: string): Promise<unknown>;
  refreshUrlsAndRetry(taskId: string): Promise<unknown>;
  cancel(taskId: string): Promise<unknown>;
  remove(taskId: string): Promise<unknown>;
  openFile(taskId: string): Promise<unknown>;
  openDir(taskId: string): Promise<unknown>;
  copySource(taskId: string): Promise<unknown>;
}

export interface TransferTaskActionDialogs {
  openScheduleDialog(taskId: string): void;
  openSpeedLimitDialog(taskId: string): void;
}

const queueActionByKind = {
  pause: 'pause',
  resume: 'resume',
  unschedule: 'unschedule',
  retry: 'retry',
  refresh_retry: 'refreshUrlsAndRetry',
  cancel: 'cancel',
  remove: 'remove',
  open_file: 'openFile',
  open_dir: 'openDir',
  copy_source: 'copySource',
} as const satisfies Partial<Record<TransferTaskAction, keyof TransferTaskActionQueue>>;

export const useTransferTaskActions = (queue: TransferTaskActionQueue, dialogs: TransferTaskActionDialogs) => {
  const runTaskAction = (taskId: string, action: TransferTaskAction) => {
    if (action === 'schedule') {
      dialogs.openScheduleDialog(taskId);
      return;
    }
    if (action === 'speed_limit') {
      dialogs.openSpeedLimitDialog(taskId);
      return;
    }

    const queueAction = queueActionByKind[action];
    void queue[queueAction](taskId);
  };

  return { runTaskAction };
};
