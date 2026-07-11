import { onActivated, onDeactivated } from 'vue';

export interface TransferPageQueue {
  list(): Promise<unknown>;
  reconcile(): Promise<unknown>;
  readonly hasInFlightTasks: boolean;
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
  let reconcileTimer: number | null = null;
  onActivated(() => {
    void queue.list();
    reconcileTimer = eventTarget.setInterval(() => {
      if (queue.hasInFlightTasks) void queue.reconcile();
    }, 2_000);
    eventTarget.addEventListener('keydown', handlers.closeContextMenuOnEscape);
    eventTarget.addEventListener('blur', handlers.closeContextMenu);
  });

  onDeactivated(() => {
    if (reconcileTimer !== null) {
      eventTarget.clearInterval(reconcileTimer);
      reconcileTimer = null;
    }
    eventTarget.removeEventListener('keydown', handlers.closeContextMenuOnEscape);
    eventTarget.removeEventListener('blur', handlers.closeContextMenu);
  });
};
