// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { optionNav } from './option-nav.action.ts';

/** The button that opened a menu, a field, and the menu's three choices. */
function menu() {
  document.body.innerHTML = `
    <button id="opener">open</button>
    <input id="field" />
    <div id="menu">
      <button role="menuitem" id="a">a</button>
      <button role="option" id="b">b</button>
      <button data-option id="c">c</button>
    </div>`;
  Element.prototype.scrollIntoView = vi.fn();
  const el = (id: string) => document.getElementById(id)!;
  return { action: optionNav(el('menu')), el };
}

function key(target: Element, k: string) {
  const e = new KeyboardEvent('keydown', { key: k, bubbles: true, cancelable: true });
  target.dispatchEvent(e);
  return e;
}

afterEach(() => {
  document.body.innerHTML = '';
});

describe('optionNav', () => {
  it('enters the choices from the opener with ↓ and moves with ↑↓, Home and End', () => {
    const { action, el } = menu();
    el('opener').focus();
    key(el('opener'), 'ArrowUp');
    expect(document.activeElement).toBe(el('opener'));
    key(el('opener'), 'ArrowDown');
    expect(document.activeElement).toBe(el('a'));
    key(el('a'), 'ArrowDown');
    key(el('b'), 'ArrowDown');
    key(el('c'), 'ArrowDown');
    expect(document.activeElement).toBe(el('c'));
    key(el('c'), 'Home');
    expect(document.activeElement).toBe(el('a'));
    key(el('a'), 'End');
    expect(document.activeElement).toBe(el('c'));
    key(el('c'), 'ArrowUp');
    expect(document.activeElement).toBe(el('b'));
    action.destroy();
  });

  it('leaves a field its keys, and a key something else already handled', () => {
    const { action, el } = menu();
    el('field').focus();
    expect(key(el('field'), 'ArrowDown').defaultPrevented).toBe(false);
    expect(document.activeElement).toBe(el('field'));
    el('opener').addEventListener('keydown', (e) => e.preventDefault());
    el('opener').focus();
    key(el('opener'), 'ArrowDown');
    expect(document.activeElement).toBe(el('opener'));
    action.destroy();
  });

  it('stops listening once the dropdown closes', () => {
    const { action, el } = menu();
    action.destroy();
    el('opener').focus();
    key(el('opener'), 'ArrowDown');
    expect(document.activeElement).toBe(el('opener'));
  });
});
