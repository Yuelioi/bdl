import { inject, onActivated, onDeactivated, onMounted, onUnmounted, type InjectionKey } from 'vue';
import type { AppTab } from '../../shared/stores/ui';

export type MobileBackHandler = () => boolean;
type RegisterMobileBack = (tab: AppTab, handler: MobileBackHandler, priority: number) => () => void;
export const mobileBackKey: InjectionKey<RegisterMobileBack> = Symbol('mobileBack');

export function useMobileBack(tab: AppTab, handler: MobileBackHandler, priority = 0) {
  const register = inject(mobileBackKey);
  let unregister: (() => void) | undefined;
  const activate = () => {
    unregister ??= register?.(tab, handler, priority);
  };
  const deactivate = () => {
    unregister?.();
    unregister = undefined;
  };
  onMounted(activate);
  onActivated(activate);
  onDeactivated(deactivate);
  onUnmounted(deactivate);
}
