// Where a popover goes: on its preferred side unless only the other side fits, then kept
// inside the window.
import type { Placement } from '#lib/types/popover.type.ts';

/** Gap kept between a popover and the window edge. */
export const VIEWPORT_MARGIN = 8;

type Rect = { left: number; right: number; top: number; bottom: number };
type Size = { width: number; height: number };

type PositionOptions = {
  /** The anchor's box. */
  rect: Rect;
  /** The popover's own size. */
  size: Size;
  placement: Placement;
  /** Gap between anchor and popover. */
  offset: number;
  viewport: Size;
};

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(value, max));
}

function pickTop({ rect, size, placement, offset, viewport }: PositionOptions): number {
  const below = rect.bottom + offset;
  const above = rect.top - offset - size.height;
  const fitsBelow = below + size.height <= viewport.height - VIEWPORT_MARGIN;
  const fitsAbove = above >= VIEWPORT_MARGIN;
  const prefersBelow = placement.startsWith('bottom');
  if (prefersBelow) {
    if (fitsBelow || !fitsAbove) return below;
    return above;
  }
  if (fitsAbove || !fitsBelow) return above;
  return below;
}

function pickLeft({ rect, size, placement }: PositionOptions): number {
  const alignsStart = placement.endsWith('start');
  if (alignsStart) return rect.left;
  return rect.right - size.width;
}

/** Top-left corner for the popover, in window coordinates. */
export function positionFor(options: PositionOptions): { left: number; top: number } {
  const { size, viewport } = options;
  const maxTop = viewport.height - VIEWPORT_MARGIN - size.height;
  const maxLeft = viewport.width - VIEWPORT_MARGIN - size.width;
  const top = clamp(pickTop(options), VIEWPORT_MARGIN, maxTop);
  const left = clamp(pickLeft(options), VIEWPORT_MARGIN, maxLeft);
  return { left, top };
}
