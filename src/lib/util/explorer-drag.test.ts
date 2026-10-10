// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest';
import { trackDrag } from './explorer-drag.ts';

const at = (type: string, x: number) => new MouseEvent(type, { clientX: x, clientY: 0 }) as PointerEvent;

describe('trackDrag', () => {
  it('ends a cancelled drag as not dropped, and stops listening', () => {
    vi.stubGlobal('requestAnimationFrame', () => 1);
    vi.stubGlobal('cancelAnimationFrame', () => {});
    const ends: boolean[] = [];
    const starts = vi.fn();
    trackDrag({ start: at('pointerdown', 0), onstart: starts, onmove: () => {}, onend: (dropped) => ends.push(dropped) });
    window.dispatchEvent(at('pointermove', 20));
    expect(starts).toHaveBeenCalledOnce();
    window.dispatchEvent(at('pointercancel', 20));
    // The next release anywhere no longer ends (and drops) the old drag.
    window.dispatchEvent(at('pointerup', 40));
    expect(ends).toEqual([false]);
  });

  it('drops on release', () => {
    vi.stubGlobal('requestAnimationFrame', () => 1);
    vi.stubGlobal('cancelAnimationFrame', () => {});
    const ends: boolean[] = [];
    trackDrag({ start: at('pointerdown', 0), onstart: () => {}, onmove: () => {}, onend: (dropped) => ends.push(dropped) });
    window.dispatchEvent(at('pointermove', 20));
    window.dispatchEvent(at('pointerup', 20));
    expect(ends).toEqual([true]);
  });
});
