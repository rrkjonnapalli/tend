// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { rowNav } from './row-nav.action.ts';

/** A search field and three rows; the second is an open folder, the third a closed one. */
function list() {
  document.body.innerHTML = `
    <main>
      <input data-row-entry aria-label="search" />
      <div data-row tabindex="-1" id="a"><button data-rename>rename</button></div>
      <div data-row tabindex="-1" id="b" aria-expanded="true"><button data-toggle>fold</button><button data-open>open</button></div>
      <div data-row tabindex="-1" id="c" aria-expanded="false"><button data-toggle>fold</button></div>
    </main>`;
  const main = document.querySelector('main')!;
  const action = rowNav(main);
  Element.prototype.scrollIntoView = vi.fn();
  return { action, el: (id: string) => document.getElementById(id)! };
}

function key(target: Element, k: string) {
  target.dispatchEvent(new KeyboardEvent('keydown', { key: k, bubbles: true, cancelable: true }));
}

afterEach(() => {
  document.body.innerHTML = '';
});

describe('rowNav', () => {
  it('goes from the search field into the rows and moves with ↑↓, Home and End', () => {
    const { el } = list();
    key(document.querySelector('input')!, 'ArrowDown');
    expect(document.activeElement).toBe(el('a'));
    key(el('a'), 'ArrowDown');
    expect(document.activeElement).toBe(el('b'));
    key(el('b'), 'End');
    expect(document.activeElement).toBe(el('c'));
    key(el('c'), 'ArrowUp');
    expect(document.activeElement).toBe(el('b'));
    key(el('b'), 'Home');
    expect(document.activeElement).toBe(el('a'));
  });

  it('presses the row controls for Enter, F2, → and ←', () => {
    const { el } = list();
    const clicks: string[] = [];
    for (const b of document.querySelectorAll('button')) b.addEventListener('click', () => clicks.push(`${b.parentElement!.id}:${b.textContent}`));
    key(el('a'), 'F2');
    key(el('b'), 'Enter');
    key(el('b'), 'ArrowLeft');
    key(el('b'), 'ArrowRight');
    key(el('c'), 'ArrowRight');
    expect(clicks).toEqual(['a:rename', 'b:open', 'b:fold', 'c:fold']);
  });

  it('leaves keys alone in other fields and after destroy', () => {
    const { action, el } = list();
    const field = document.createElement('input');
    el('a').append(field);
    key(field, 'ArrowDown');
    expect(document.activeElement).not.toBe(el('b'));
    action.destroy();
    key(document.querySelector('input')!, 'ArrowDown');
    expect(document.activeElement).not.toBe(el('a'));
  });
});
