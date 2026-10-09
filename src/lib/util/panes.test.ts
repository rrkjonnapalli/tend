// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import { focusDock, focusMain, focusSide } from './panes.ts';

describe('panes', () => {
  it('goes to the open item on the side and back to where the main content was left', () => {
    document.body.innerHTML = `
      <aside data-pane="side"><div data-row tabindex="-1" id="a"></div><div data-row tabindex="-1" id="b" aria-selected="true"></div></aside>
      <main><input id="search" /><div data-zone="rows"><a href="/1" data-row id="r1">one</a></div></main>`;
    expect(focusSide(document)).toBe(true);
    expect(document.activeElement?.id).toBe('b');
    expect(focusMain(document, null)).toBe(true);
    expect(document.activeElement?.id).toBe('r1');
    const search = document.getElementById('search')!;
    expect(focusMain(document, search)).toBe(true);
    expect(document.activeElement).toBe(search);
  });

  it('says when there is no side pane', () => {
    document.body.innerHTML = '<main></main>';
    expect(focusSide(document)).toBe(false);
  });

  it('goes to the dock on the current section', () => {
    document.body.innerHTML = `
      <nav data-pane="dock"><a href="/notes" id="notes">n</a><a href="/tasks" id="tasks" aria-current="page">t</a><button id="gear">s</button></nav>`;
    expect(focusDock(document)).toBe(true);
    expect(document.activeElement?.id).toBe('tasks');
  });
});
