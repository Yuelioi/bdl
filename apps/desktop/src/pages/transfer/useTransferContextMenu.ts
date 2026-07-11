import { computed, ref, type Ref } from 'vue';

import type { TaskActionDescriptor, TaskActionKind, TransferTaskView } from '../../stores/transferView';

export interface ContextMenuPosition {
  taskId: string;
  x: number;
  y: number;
}

export interface ContextMenuViewport {
  width: number;
  height: number;
}

const MENU_WIDTH = 190;
const MENU_HEIGHT = 260;
const VIEWPORT_GUTTER = 8;

export const constrainContextMenuPosition = (
  taskId: string,
  clientX: number,
  clientY: number,
  viewport: ContextMenuViewport,
): ContextMenuPosition => ({
  taskId,
  x: Math.max(VIEWPORT_GUTTER, Math.min(clientX, viewport.width - MENU_WIDTH - VIEWPORT_GUTTER)),
  y: Math.max(VIEWPORT_GUTTER, Math.min(clientY, viewport.height - MENU_HEIGHT - VIEWPORT_GUTTER)),
});

export const useTransferContextMenu = (
  taskViews: Readonly<Ref<TransferTaskView[]>>,
  runTaskAction: (taskId: string, action: Exclude<TaskActionKind, 'none'>) => void,
) => {
  const contextMenu = ref<ContextMenuPosition | null>(null);
  const contextTaskView = computed(() => taskViews.value.find((view) => view.id === contextMenu.value?.taskId) ?? null);
  const contextActions = computed<TaskActionDescriptor[]>(() => {
    const view = contextTaskView.value;
    if (!view) return [];
    const actions = [...view.secondaryActions];
    if (view.primaryAction !== 'none') {
      actions.unshift({ kind: view.primaryAction, label: view.primaryActionLabel, icon: view.primaryActionIcon });
    }
    return actions;
  });

  const openContextMenu = (taskId: string, event: MouseEvent) => {
    contextMenu.value = constrainContextMenuPosition(taskId, event.clientX, event.clientY, {
      width: window.innerWidth,
      height: window.innerHeight,
    });
  };
  const closeContextMenu = () => {
    contextMenu.value = null;
  };
  const closeContextMenuOnEscape = (event: KeyboardEvent) => {
    if (event.key === 'Escape') closeContextMenu();
  };
  const runContextAction = (action: Exclude<TaskActionKind, 'none'>) => {
    const taskId = contextMenu.value?.taskId;
    closeContextMenu();
    if (taskId) runTaskAction(taskId, action);
  };

  return {
    contextMenu,
    contextTaskView,
    contextActions,
    openContextMenu,
    closeContextMenu,
    closeContextMenuOnEscape,
    runContextAction,
  };
};

export const contextIcon = (icon: string): string => {
  const aliases: Record<string, string> = {
    pause: 'player-pause',
    play: 'player-play',
    refresh: 'refresh',
    x: 'x',
    trash: 'trash',
    file: 'file',
    folder: 'folder',
    copy: 'copy',
  };
  return `i-tabler-${aliases[icon] ?? icon}`;
};
