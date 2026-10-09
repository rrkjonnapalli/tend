// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import { startTabbingIn } from './focus-start.ts';

describe('startTabbingIn', () => {
  it('moves focus into the element and leaves its tabindex as it was', () => {
    document.body.innerHTML = '<button id="close">x</button><div id="page"></div><div id="kept" tabindex="0"></div>';
    const page = document.getElementById('page')!;
    startTabbingIn(page);
    expect(document.activeElement).toBe(page);
    expect(page.hasAttribute('tabindex')).toBe(false);
    const kept = document.getElementById('kept')!;
    startTabbingIn(kept);
    expect(kept.getAttribute('tabindex')).toBe('0');
  });
});
