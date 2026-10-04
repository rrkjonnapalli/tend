import { describe, expect, it } from 'vitest';
import { positionFor } from './popover-position.ts';

const viewport = { width: 800, height: 600 };
const size = { width: 200, height: 100 };

describe('positionFor', () => {
  it('opens below the anchor, aligned to its start', () => {
    const rect = { left: 100, right: 150, top: 50, bottom: 70 };
    expect(positionFor({ rect, size, placement: 'bottom-start', offset: 4, viewport })).toEqual({ left: 100, top: 74 });
  });

  it('flips above when it does not fit below', () => {
    const rect = { left: 100, right: 150, top: 550, bottom: 570 };
    expect(positionFor({ rect, size, placement: 'bottom-end', offset: 4, viewport })).toEqual({ left: 8, top: 446 });
  });

  it('stays inside the window', () => {
    const rect = { left: 790, right: 790, top: 10, bottom: 10 };
    expect(positionFor({ rect, size, placement: 'top-start', offset: 4, viewport })).toEqual({ left: 592, top: 14 });
  });
});
