import { onActivated, onDeactivated } from 'vue';

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
  onActivated(() => {
    void queue.list();
    eventTarget.addEventListener('keydown', handlers.closeContextMenuOnEscape);
    eventTarget.addEventListener('blur', handlers.closeContextMenu);
  });

  onDeactivated(() => {
    eventTarget.removeEventListener('keydown', handlers.closeContextMenuOnEscape);
    eventTarget.removeEventListener('blur', handlers.closeContextMenu);
  });
};
