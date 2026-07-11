import { computed, ref } from 'vue';

import type { DownloadTask } from '../../api/dto';
import { scheduledLocalError, toDateTimeLocalValue, toScheduledIso } from '../../utils/schedule';
import { speedLimitMibError, toBytesPerSecond, toMibPerSecondInput } from '../../utils/speedLimit';

export interface TransferDialogQueue {
  tasks: DownloadTask[];
  schedule(taskId: string, scheduledAt: string): Promise<boolean>;
  setSpeedLimit(taskId: string, speedLimitBytesPerSecond?: number): Promise<boolean>;
}

export const useTransferDialogs = (queue: TransferDialogQueue, now: () => number = Date.now) => {
  const scheduleDialogOpen = ref(false);
  const scheduleTaskId = ref<string | null>(null);
  const scheduleLocal = ref('');
  const scheduleMin = ref('');
  const scheduleValidationNow = ref(now());
  const speedLimitDialogOpen = ref(false);
  const speedLimitTaskId = ref<string | null>(null);
  const speedLimitMib = ref('');

  const scheduleError = computed(() => scheduledLocalError(scheduleLocal.value, scheduleValidationNow.value, true));
  const speedLimitError = computed(() => speedLimitMibError(speedLimitMib.value));

  const openScheduleDialog = (taskId: string) => {
    const currentTime = now();
    const task = queue.tasks.find((candidate) => candidate.id === taskId);
    scheduleValidationNow.value = currentTime;
    scheduleTaskId.value = taskId;
    scheduleMin.value = toDateTimeLocalValue(new Date(currentTime + 60_000));
    scheduleLocal.value = toDateTimeLocalValue(
      task?.scheduled_at ? new Date(task.scheduled_at) : new Date(currentTime + 300_000),
    );
    scheduleDialogOpen.value = true;
  };

  const submitSchedule = async () => {
    scheduleValidationNow.value = now();
    if (!scheduleTaskId.value || scheduleError.value) return false;
    const updated = await queue.schedule(scheduleTaskId.value, toScheduledIso(scheduleLocal.value));
    if (updated) scheduleDialogOpen.value = false;
    return Boolean(updated);
  };

  const openSpeedLimitDialog = (taskId: string) => {
    const task = queue.tasks.find((candidate) => candidate.id === taskId);
    speedLimitTaskId.value = taskId;
    speedLimitMib.value = toMibPerSecondInput(task?.speed_limit_bytes_per_second);
    speedLimitDialogOpen.value = true;
  };

  const submitSpeedLimit = async () => {
    if (!speedLimitTaskId.value || speedLimitError.value) return false;
    const updated = await queue.setSpeedLimit(speedLimitTaskId.value, toBytesPerSecond(speedLimitMib.value));
    if (updated) speedLimitDialogOpen.value = false;
    return Boolean(updated);
  };

  return {
    scheduleDialogOpen,
    scheduleLocal,
    scheduleMin,
    scheduleError,
    speedLimitDialogOpen,
    speedLimitMib,
    speedLimitError,
    openScheduleDialog,
    submitSchedule,
    openSpeedLimitDialog,
    submitSpeedLimit,
  };
};
