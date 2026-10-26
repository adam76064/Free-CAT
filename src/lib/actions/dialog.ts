// src/lib/actions/dialog.ts
// Svelte action that turns a modal overlay into an accessible dialog:
//  - role="dialog", aria-modal and an accessible name
//  - moves focus inside on open (unless something inside already has it, e.g. an autofocus input)
//  - traps Tab / Shift+Tab inside the dialog
//  - Escape clicks the dialog's Cancel/Close button (or [data-dialog-close]) when the
//    component hasn't already handled the key itself
//  - restores focus to whatever was focused before it opened
//
// Usage: <div class="fixed inset-0 ..." use:dialog={{ label: 'Concordance search' }}>

export interface DialogOptions {
  /** Accessible name, used when the element has no aria-label/aria-labelledby. */
  label?: string;
  /** Set false to disable the Escape-to-close behavior. */
  closeOnEscape?: boolean;
}

const FOCUSABLE = [
  'a[href]',
  'button:not([disabled])',
  'input:not([disabled]):not([type="hidden"])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[contenteditable="true"]',
  '[tabindex]:not([tabindex="-1"])',
].join(',');

function isVisible(el: HTMLElement): boolean {
  if (el.hasAttribute('hidden') || el.closest('[hidden]')) return false;
  return getComputedStyle(el).display !== 'none';
}

export function focusableElements(root: HTMLElement): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(isVisible);
}

const CLOSE_TEXT = /^(cancel|close|dismiss|×|✕|✖|x)$/i;
const CLOSE_LABEL = /^(close|cancel|dismiss)\b/i;

export function findCloseButton(root: HTMLElement): HTMLElement | null {
  const explicit = root.querySelector<HTMLElement>('[data-dialog-close]');
  if (explicit) return explicit;
  for (const btn of Array.from(root.querySelectorAll<HTMLButtonElement>('button:not([disabled])'))) {
    const text = (btn.textContent ?? '').trim();
    const label = btn.getAttribute('aria-label') ?? btn.getAttribute('title') ?? '';
    if (CLOSE_TEXT.test(text) || CLOSE_LABEL.test(label)) return btn;
  }
  return null;
}

export function dialog(node: HTMLElement, initial: DialogOptions = {}) {
  let opts = initial;
  const previouslyFocused = document.activeElement as HTMLElement | null;

  if (!node.hasAttribute('role')) node.setAttribute('role', 'dialog');
  node.setAttribute('aria-modal', 'true');
  if (opts.label && !node.hasAttribute('aria-label') && !node.hasAttribute('aria-labelledby')) {
    node.setAttribute('aria-label', opts.label);
  }

  // Let the component's own autofocus run first, then fall back to the first control.
  queueMicrotask(() => {
    if (!node.isConnected || node.contains(document.activeElement)) return;
    const first = focusableElements(node)[0];
    if (first) first.focus();
    else {
      node.setAttribute('tabindex', '-1');
      node.focus();
    }
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Tab') {
      const items = focusableElements(node);
      if (items.length === 0) {
        e.preventDefault();
        return;
      }
      const first = items[0];
      const last = items[items.length - 1];
      const active = document.activeElement as HTMLElement | null;
      const inside = !!active && node.contains(active);
      if (e.shiftKey && (!inside || active === first)) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && (!inside || active === last)) {
        e.preventDefault();
        first.focus();
      }
    } else if (e.key === 'Escape' && !e.defaultPrevented && opts.closeOnEscape !== false) {
      const close = findCloseButton(node);
      if (close) {
        e.preventDefault();
        e.stopPropagation(); // don't also trigger global Escape shortcuts underneath
        close.click();
      }
    }
  }

  node.addEventListener('keydown', onKeydown);

  return {
    update(next: DialogOptions) {
      opts = next;
    },
    destroy() {
      node.removeEventListener('keydown', onKeydown);
      if (previouslyFocused && previouslyFocused.isConnected) previouslyFocused.focus?.();
    },
  };
}
