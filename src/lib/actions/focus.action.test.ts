// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import { focusIfIdle } from './focus.action.ts';

describe('focusIfIdle', () => {
  it('takes focus only when nothing else has it', () => {
    document.body.innerHTML = '<button id="view">All</button><input id="add" />';
    const add = document.getElementById('add')!;
    focusIfIdle(add, true);
    expect(document.activeElement).toBe(add);
    const view = document.getElementById('view')!;
    view.focus();
    focusIfIdle(add, true);
    expect(document.activeElement).toBe(view);
  });
});
