/**
 * toast.ts — Global in-app toast/notification store.
 * Replaces all browser-native alert() calls with non-blocking
 * slide-in toast notifications stacked in the top-right corner.
 */
import { writable } from 'svelte/store';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
  id: string;
  type: ToastType;
  title: string;
  message?: string;
  duration: number;
}

function createToastStore() {
  const { subscribe, update } = writable<Toast[]>([]);

  function push(opts: Omit<Toast, 'id'>) {
    const id = crypto.randomUUID();
    const entry: Toast = { ...opts, id };
    update(t => [entry, ...t]);
    setTimeout(() => dismiss(id), entry.duration);
    return id;
  }

  function dismiss(id: string) {
    update(t => t.filter(x => x.id !== id));
  }

  return {
    subscribe,
    success: (title: string, message?: string) =>
      push({ type: 'success', title, message, duration: 4000 }),
    error: (title: string, message?: string) =>
      push({ type: 'error', title, message, duration: 7000 }),
    warning: (title: string, message?: string) =>
      push({ type: 'warning', title, message, duration: 5000 }),
    info: (title: string, message?: string) =>
      push({ type: 'info', title, message, duration: 4000 }),
    dismiss,
  };
}

export const toast = createToastStore();
