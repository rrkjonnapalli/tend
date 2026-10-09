// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { zoneNav } from './zone-nav.action.ts';

/** A page bar, a view line, an empty list, a search field and a list of two rows. */
function page() {
  document.body.innerHTML = `
    <main>
      <div data-zone="bar"><a href="/x" id="crumb">notes</a></div>
      <nav data-zone="bar">
        <button id="v1" data-pick aria-pressed="false">My day</button>
        <button id="v2" data-pick aria-pressed="true">All</button>
        <button id="tog">hide</button>
      </nav>
      <div data-zone="rows" id="empty"></div>
      <label data-zone="field"><input id="search" /></label>
      <div data-zone="rows"><a href="/1" data-row id="r1">one</a><a href="/2" data-row id="r2">two</a></div>
    </main>`;
  // jsdom lays nothing out; treat every element as shown.
  Element.prototype.getClientRects = () => [{}] as unknown as DOMRectList;
  Element.prototype.scrollIntoView = vi.fn();
  const action = zoneNav(document.querySelector('main')!);
  return { action, el: (id: string) => document.getElementById(id)! };
}

function key(target: Element, k: string) {
  target.dispatchEvent(new KeyboardEvent('keydown', { key: k, bubbles: true, cancelable: true }));
}

afterEach(() => {
  document.body.innerHTML = '';
});

describe('zoneNav', () => {
  it('walks the zones with ↑↓, skipping empty ones, and leaves a list only from its first row', () => {
    const { el } = page();
    el('r2').focus();
    key(el('r2'), 'ArrowUp');
    expect(document.activeElement).toBe(el('r1'));
    key(el('r1'), 'ArrowUp');
    expect(document.activeElement).toBe(el('search'));
    key(el('search'), 'ArrowUp');
    // The view line is entered on the current view.
    expect(document.activeElement).toBe(el('v2'));
    key(el('v2'), 'ArrowUp');
    expect(document.activeElement).toBe(el('crumb'));
    key(el('crumb'), 'ArrowDown');
    key(el('v2'), 'ArrowDown');
    expect(document.activeElement).toBe(el('search'));
    key(el('search'), 'ArrowDown');
    expect(document.activeElement).toBe(el('r1'));
  });

  it('moves along a bar with ←→ and presses the views it lands on', () => {
    const { el } = page();
    const picked: string[] = [];
    for (const id of ['v1', 'v2', 'tog']) el(id).addEventListener('click', () => picked.push(id));
    el('v2').focus();
    key(el('v2'), 'ArrowLeft');
    expect(document.activeElement).toBe(el('v1'));
    key(el('v1'), 'ArrowRight');
    key(el('v2'), 'ArrowRight');
    expect(document.activeElement).toBe(el('tog'));
    // Only data-pick controls are pressed.
    expect(picked).toEqual(['v1', 'v2']);
  });

  it('leaves ←→ in fields alone', () => {
    const { el } = page();
    el('search').focus();
    key(el('search'), 'ArrowLeft');
    expect(document.activeElement).toBe(el('search'));
  });
});
