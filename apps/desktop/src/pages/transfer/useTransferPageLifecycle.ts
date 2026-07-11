import { onBeforeUnmount, onMounted } from 'vue';

export interface TransferPageQueue {
  list(): Promise<unknown>;
}

export interface TransferPageLifecycleHandlers {
  closeContextMenu(): void;
  closeContextMenuOnEscape(event: KeyboardEvent): void;
}

export const useTransferPageLifecycle = (
  queue: TransferPageQueue,
  handlers: TransferPageLifecycleHandlers,
  eventTarget: Window = window,
) => {
  onMounted(() => {
    void queue.list();
    eventTarget.addEventListener('keydown', handlers.closeContextMenuOnEscape);
    eventTarget.addEventListener('blur', handlers.closeContextMenu);
  });

  onBeforeUnmount(() => {
    eventTarget.removeEventListener('keydown', handlers.closeContextMenuOnEscape);
    eventTarget.removeEventListener('blur', handlers.closeContextMenu);
  });
};
