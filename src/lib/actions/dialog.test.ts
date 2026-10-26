// @vitest-environment happy-dom
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { dialog, findCloseButton, focusableElements } from './dialog';

function mount(html: string, opts = {}) {
  document.body.innerHTML = `<button id="opener">open</button>${html}`;
  const node = document.getElementById('dlg') as HTMLElement;
  const opener = document.getElementById('opener') as HTMLElement;
  opener.focus();
  const action = dialog(node, opts);
  return { node, opener, action };
}
const key = (node: HTMLElement, k: string, shiftKey = false) => {
  const e = new KeyboardEvent('keydown', { key: k, shiftKey, bubbles: true, cancelable: true });
  node.dispatchEvent(e);
  return e;
};
const flush = () => new Promise((r) => setTimeout(r, 0));

describe('dialog action', () => {
  beforeEach(() => { document.body.innerHTML = ''; });

  it('adds dialog semantics and an accessible name, without overriding existing ones', () => {
    const { node } = mount('<div id="dlg"><button>ok</button></div>', { label: 'Concordance' });
    expect(node.getAttribute('role')).toBe('dialog');
    expect(node.getAttribute('aria-modal')).toBe('true');
    expect(node.getAttribute('aria-label')).toBe('Concordance');

    const m2 = mount('<div id="dlg" aria-label="Mine" role="alertdialog"></div>', { label: 'Other' });
    expect(m2.node.getAttribute('aria-label')).toBe('Mine');
    expect(m2.node.getAttribute('role')).toBe('alertdialog');
  });

  it('moves focus to the first control, but respects an element that already has focus', async () => {
    const a = mount('<div id="dlg"><input id="i1"><button id="b1">go</button></div>');
    await flush();
    expect(document.activeElement?.id).toBe('i1');
    a.action.destroy();

    document.body.innerHTML = '<button id="opener">o</button><div id="dlg"><input id="i1"><input id="i2"></div>';
    (document.getElementById('i2') as HTMLElement).focus();
    dialog(document.getElementById('dlg')!);
    await flush();
    expect(document.activeElement?.id).toBe('i2');
  });

  it('traps Tab and Shift+Tab inside the dialog', async () => {
    const { node } = mount('<div id="dlg"><button id="a">a</button><button id="b">b</button></div>');
    await flush();
    const b = document.getElementById('b')!;
    b.focus();
    expect(key(node, 'Tab').defaultPrevented).toBe(true);
    expect(document.activeElement?.id).toBe('a');
    expect(key(node, 'Tab', true).defaultPrevented).toBe(true);
    expect(document.activeElement?.id).toBe('b');
    // middle of the dialog: native Tab behaviour is left alone
    document.getElementById('a')!.focus();
    const mid = mount('<div id="dlg"><button id="x">x</button><button id="y">y</button><button id="z">z</button></div>');
    await flush();
    document.getElementById('y')!.focus();
    expect(key(mid.node, 'Tab').defaultPrevented).toBe(false);
  });

  it('Escape clicks the Cancel/Close button, and stops the event from reaching global handlers', () => {
    const onCancel = vi.fn();
    const { node } = mount('<div id="dlg"><button id="save">Save</button><button id="c">Cancel</button></div>');
    document.getElementById('c')!.addEventListener('click', onCancel);
    const globalHandler = vi.fn();
    document.body.addEventListener('keydown', globalHandler);
    const e = key(node, 'Escape');
    expect(onCancel).toHaveBeenCalledTimes(1);
    expect(e.defaultPrevented).toBe(true);
    expect(globalHandler).not.toHaveBeenCalled();
  });

  it('Escape does nothing if the component already handled it, or if there is no close button', () => {
    const onClose = vi.fn();
    const { node } = mount('<div id="dlg"><button id="c">Close</button></div>');
    document.getElementById('c')!.addEventListener('click', onClose);
    node.addEventListener('keydown', (e) => e.preventDefault(), { capture: true });
    key(node, 'Escape');
    expect(onClose).not.toHaveBeenCalled();

    const none = mount('<div id="dlg"><button>Save</button></div>');
    expect(key(none.node, 'Escape').defaultPrevented).toBe(false);
  });

  it('finds close buttons by text, aria-label/title, or [data-dialog-close]', () => {
    const q = (h: string) => { document.body.innerHTML = `<div id="dlg">${h}</div>`; return findCloseButton(document.getElementById('dlg')!); };
    expect(q('<button>✕</button>')?.textContent).toBe('✕');
    expect(q('<button aria-label="Close dialog">🗙</button>')).not.toBeNull();
    expect(q('<button title="Cancel">←</button>')).not.toBeNull();
    expect(q('<button>Save</button><span data-dialog-close id="x"></span>')?.id).toBe('x');
    expect(q('<button>Save changes</button>')).toBeNull();
  });

  it('restores focus to the opener on destroy and ignores hidden controls', async () => {
    const { opener, action } = mount('<div id="dlg"><button hidden id="h">h</button><button id="v">v</button></div>');
    await flush();
    expect(document.activeElement?.id).toBe('v');
    expect(focusableElements(document.getElementById('dlg')!).map((e) => e.id)).toEqual(['v']);
    action.destroy();
    expect(document.activeElement).toBe(opener);
  });
});
