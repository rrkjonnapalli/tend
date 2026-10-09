// Arrow keys across a page, zone by zone, top to bottom (see decisions §12). A zone is any element
// with `data-zone`:
// - `bar`: a horizontal line of controls (the page bar, the task views). ← → move along it; a
//   control marked `data-pick` is pressed as the focus lands on it (switching a view).
// - `field`: a one-line field (search, add). Typing keys stay with it.
// - `rows`: a list of `data-row` rows (see rowNav): ↑↓ move, Enter opens, F2 renames.
// - `board`: the task board, which moves its own cards; ↑ from its top card leaves it.
// ↑ and ↓ leave a zone for the one above or below (↑ only from a list's first row); zones with
// nothing to focus are skipped.
import { rowKey } from './row-nav.action.ts';

const ZONE = '[data-zone]';
const CONTROL = 'button:not(:disabled), a[href], [tabindex]:not([tabindex="-1"])';

const isShown = (el: HTMLElement) => el.getClientRects().length > 0;

function controlsOf(zone: HTMLElement): HTMLElement[] {
  return [...zone.querySelectorAll<HTMLElement>(CONTROL)].filter(isShown);
}

/** Where focus lands when a zone is entered; null when the zone has nothing to focus. */
function entryOf(zone: HTMLElement): HTMLElement | null {
  const kind = zone.dataset.zone;
  if (kind === 'rows') return zone.querySelector<HTMLElement>('[data-row]');
  if (kind === 'board') return zone.querySelector<HTMLElement>('[data-card]');
  if (kind === 'field') return zone.querySelector<HTMLElement>('input, textarea');
  const controls = controlsOf(zone);
  const current = controls.find((c) => c.matches('[aria-pressed="true"][data-pick], [aria-current="page"]'));
  return current ?? controls[0] ?? null;
}

function focus(el: HTMLElement) {
  el.focus();
  el.scrollIntoView({ block: 'nearest' });
}

/** Focuses the first zone with something to focus, from `from` stepping by `step`. */
function enterFrom(zones: HTMLElement[], from: number, step: number): boolean {
  for (let i = from; i >= 0 && i < zones.length; i += step) {
    const entry = entryOf(zones[i]);
    if (entry) {
      focus(entry);
      return true;
    }
  }
  return false;
}

/** ← → along a bar; a `data-pick` control is pressed as it gets focus. */
function alongBar(zone: HTMLElement, target: HTMLElement, key: string): boolean {
  const controls = controlsOf(zone);
  const at = controls.indexOf(target);
  if (at < 0) return false;
  const next = key === 'ArrowRight' ? at + 1 : at - 1;
  const control = controls[next];
  if (!control) return true;
  focus(control);
  if (control.hasAttribute('data-pick')) control.click();
  return true;
}

/** True when ↑ should leave a rows zone: the focus is on its first row. */
function atTopRow(zone: HTMLElement, target: HTMLElement): boolean {
  const first = zone.querySelector<HTMLElement>('[data-row]');
  return first !== null && target === first;
}

export function zoneNav(node: HTMLElement) {
  function onkeydown(e: KeyboardEvent) {
    const hasModifier = e.metaKey || e.ctrlKey || e.altKey || e.shiftKey;
    if (e.defaultPrevented || hasModifier) return;
    const target = e.target as HTMLElement;
    const zone = target.closest<HTMLElement>(ZONE);
    if (!zone || !node.contains(zone)) return;
    const kind = zone.dataset.zone;
    const isVertical = e.key === 'ArrowUp' || e.key === 'ArrowDown';
    const leavingRows = kind === 'rows' && e.key === 'ArrowUp' && atTopRow(zone, target);
    if (kind === 'rows' && !leavingRows) {
      rowKey(zone, e);
      return;
    }
    if (kind === 'bar' && (e.key === 'ArrowLeft' || e.key === 'ArrowRight')) {
      if (alongBar(zone, target, e.key)) e.preventDefault();
      return;
    }
    if (!isVertical) return;
    const zones = [...node.querySelectorAll<HTMLElement>(ZONE)].filter(isShown);
    const at = zones.indexOf(zone);
    const step = e.key === 'ArrowDown' ? 1 : -1;
    if (enterFrom(zones, at + step, step)) e.preventDefault();
  }

  node.addEventListener('keydown', onkeydown);
  return {
    destroy() {
      node.removeEventListener('keydown', onkeydown);
    },
  };
}
