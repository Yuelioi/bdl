import { computed } from 'vue';
import { useQueueStore } from '../stores/queue';
import type { AppTab } from '../stores/ui';

export const navItems: Array<{ value: AppTab; label: string; description: string; icon: string; shortcut: string }> = [
  { value: 'parse', label: '解析', description: '添加与选择', icon: 'i-tabler-link', shortcut: '1' },
  { value: 'library', label: '内容库', description: '收藏与订阅', icon: 'i-tabler-books', shortcut: '2' },
  { value: 'transfer', label: '传输', description: '队列与恢复', icon: 'i-tabler-transfer', shortcut: '3' },
  { value: 'settings', label: '设置', description: '偏好与维护', icon: 'i-tabler-adjustments', shortcut: '4' },
  { value: 'about', label: '关于', description: '版本与链接', icon: 'i-tabler-info-circle', shortcut: '5' },
];

export const useNavigationStatus = () => {
  const queue = useQueueStore();
  const transferBadgeCount = computed(
    () => queue.tasks.filter((task) => task.status !== 'completed' && task.status !== 'cancelled').length,
  );
  const attentionCount = computed(
    () => queue.tasks.filter((task) => task.status === 'failed' || task.status === 'cancelled').length,
  );
  const scheduledTaskCount = computed(
    () =>
      queue.tasks.filter(
        (task) => task.status === 'waiting' && task.scheduled_at && Date.parse(task.scheduled_at) > Date.now(),
      ).length,
  );
  const queueHealthLabel = computed(() => {
    if (queue.loading) return '同步队列';
    if (attentionCount.value > 0) return `${attentionCount.value} 项需处理`;
    if (scheduledTaskCount.value === transferBadgeCount.value && scheduledTaskCount.value > 0) {
      return `${scheduledTaskCount.value} 项已定时`;
    }
    if (transferBadgeCount.value > 0) return `${transferBadgeCount.value} 项进行中`;
    return '队列空闲';
  });
  const aggregateSpeedLabel = computed(() => {
    const bytesPerSecond = queue.totalSpeedBytesPerSecond();
    if (bytesPerSecond <= 0 || !Number.isFinite(bytesPerSecond)) return '0 B/s';
    const units = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
    let value = bytesPerSecond;
    let unitIndex = 0;
    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex += 1;
    }
    return `${value >= 100 || unitIndex === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[unitIndex]}`;
  });
  return { transferBadgeCount, attentionCount, queueHealthLabel, aggregateSpeedLabel };
};
