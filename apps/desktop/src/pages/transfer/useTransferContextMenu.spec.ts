import { ref } from 'vue';
import { describe, expect, it, vi } from 'vitest';

import type { TransferTaskView } from '../../stores/transferView';
import { constrainContextMenuPosition, useTransferContextMenu } from './useTransferContextMenu';

const view = {
  id: 'task:one',
  displayTitle: '任务一',
  primaryAction: 'pause',
  primaryActionLabel: '暂停',
  primaryActionIcon: 'pause',
  secondaryActions: [{ kind: 'open_dir', label: '打开目录', icon: 'folder' }],
} as TransferTaskView;

describe('transfer context menu', () => {
  it('keeps the menu inside the viewport', () => {
    expect(constrainContextMenuPosition('task:one', 1000, 700, { width: 1024, height: 720 })).toEqual({
      taskId: 'task:one',
      x: 826,
      y: 452,
    });
  });

  it('places the primary action before secondary actions', () => {
    const menu = useTransferContextMenu(ref([view]), vi.fn());
    menu.contextMenu.value = { taskId: view.id, x: 8, y: 8 };
    expect(menu.contextActions.value.map((action) => action.kind)).toEqual(['pause', 'open_dir']);
  });

  it('closes before dispatching a selected action', () => {
    const run = vi.fn();
    const menu = useTransferContextMenu(ref([view]), run);
    menu.contextMenu.value = { taskId: view.id, x: 8, y: 8 };
    menu.runContextAction('pause');
    expect(menu.contextMenu.value).toBeNull();
    expect(run).toHaveBeenCalledWith('task:one', 'pause');
  });
});
