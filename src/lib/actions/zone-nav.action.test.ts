// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { zoneNav } from './zone-nav.action.ts';
import { pushLayer } from '#lib/util/layers.ts';

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

  it('walks a field grid line by line, and ← → along a line', () => {
    document.body.innerHTML = `
      <main>
        <textarea data-zone="field" id="title">Ship it</textarea>
        <div data-zone="props">
          <div data-prop><button id="status">Not started</button></div>
          <div data-prop><button id="due">Today</button><button id="time">9:00</button></div>
          <div data-prop><input id="name" value="Ann" /></div>
        </div>
        <div data-zone="rows"><div data-row tabindex="-1" id="sub"><button id="sub-edit">edit</button></div></div>
      </main>`;
    zoneNav(document.querySelector('main')!);
    const el = (id: string) => document.getElementById(id)! as HTMLElement;
    const title = el('title') as HTMLTextAreaElement;
    title.focus();
    title.setSelectionRange(0, 0);
    key(title, 'ArrowDown');
    // The caret isn't at the end of the title, so ↓ stays in it.
    expect(document.activeElement).toBe(title);
    title.setSelectionRange(7, 7);
    key(title, 'ArrowDown');
    expect(document.activeElement).toBe(el('status'));
    key(el('status'), 'ArrowDown');
    expect(document.activeElement).toBe(el('due'));
    key(el('due'), 'ArrowRight');
    expect(document.activeElement).toBe(el('time'));
    key(el('time'), 'ArrowDown');
    expect(document.activeElement).toBe(el('name'));
    key(el('name'), 'ArrowDown');
    expect(document.activeElement).toBe(el('sub'));
    key(el('sub'), 'ArrowRight');
    expect(document.activeElement).toBe(el('sub-edit'));
    key(el('sub-edit'), 'ArrowLeft');
    expect(document.activeElement).toBe(el('sub'));
  });

  it('leaves every arrow to an open dropdown', () => {
    const { el } = page();
    el('r1').focus();
    const close = pushLayer(() => {});
    key(el('r1'), 'ArrowDown');
    expect(document.activeElement).toBe(el('r1'));
    close();
    key(el('r1'), 'ArrowDown');
    expect(document.activeElement).toBe(el('r2'));
  });

  it('leaves a field inside a bar with ← at its start and → at its end, and keeps typing keys otherwise', () => {
    document.body.innerHTML = `
      <main>
        <div data-zone="bar">
          <button id="tab" data-pick>Words</button>
          <button id="add">+</button>
          <div data-zone="field"><input id="find" value="ab" /></div>
        </div>
      </main>`;
    Element.prototype.getClientRects = () => [{}] as unknown as DOMRectList;
    Element.prototype.scrollIntoView = vi.fn();
    zoneNav(document.querySelector('main')!);
    const find = document.getElementById('find') as HTMLInputElement;
    find.focus();
    find.setSelectionRange(1, 1);
    key(find, 'ArrowLeft');
    expect(document.activeElement).toBe(find);
    find.setSelectionRange(0, 0);
    key(find, 'ArrowLeft');
    expect(document.activeElement).toBe(document.getElementById('add'));
    key(document.getElementById('add')!, 'ArrowRight');
    expect(document.activeElement).toBe(find);
  });
});
