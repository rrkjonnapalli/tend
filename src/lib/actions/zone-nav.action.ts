// Arrow keys across a page, zone by zone, top to bottom (see decisions §12). A zone is any element
// with `data-zone`:
// - `bar`: a horizontal line of controls (page bar, task views, a section's head). ← → move along
//   it; a control marked `data-pick` is pressed as the focus lands on it (switching a view).
// - `field`: a field (search, add, a title). Typing keys stay with it.
// - `rows`: `data-row` rows (see rowNav): ↑↓ move, Enter opens, F2 renames, ← → step through the
//   row's own controls (or fold an explorer folder).
// - `board`: the task board, which moves its own cards; ↑ from a top card leaves it.
// - `props`: a grid of `data-prop` lines (a task's or person's fields): ↑↓ between lines, ← →
//   between the controls on a line.
// - `stack`: controls one under another (attachments, comments): ↑↓ between them.
// ↑ / ↓ past a zone's first or last item go to the zone above or below; zones with nothing to
// focus are skipped. In a field ← → are for typing, and ↑ ↓ leave a multi-line field only from
// its first or last character; a rich editor (the Markdown editor) keeps all its arrows.
import { rowKey } from './row-nav.action.ts';
import { isTypingTarget } from '#lib/util/keys.ts';
import { hasLayers } from '#lib/util/layers.ts';

const ZONE = '[data-zone]';
const CONTROL =
  'button:not(:disabled), a[href], input:not(:disabled), textarea, [contenteditable="true"], [tabindex]:not([tabindex="-1"])';

const isShown = (el: HTMLElement) => el.getClientRects().length > 0 && getComputedStyle(el).visibility !== 'hidden';

/** The focusable controls in `el` (itself included when it is one), in page order. */
function controlsOf(el: HTMLElement): HTMLElement[] {
  const own = el.matches(CONTROL) ? [el] : [];
  return [...own, ...el.querySelectorAll<HTMLElement>(CONTROL)].filter(isShown);
}

/** The zone's items for ↑↓, each focused through its first control. */
function itemsOf(zone: HTMLElement): HTMLElement[] {
  const kind = zone.dataset.zone;
  if (kind === 'rows') return [...zone.querySelectorAll<HTMLElement>('[data-row]')];
  if (kind === 'props') return [...zone.querySelectorAll<HTMLElement>('[data-prop]')].filter((p) => controlsOf(p).length > 0);
  if (kind === 'stack') return controlsOf(zone);
  return [];
}

/** Where focus lands when a zone is entered, from above (`step` 1) or below (-1). */
function entryOf(zone: HTMLElement, step: number): HTMLElement | null {
  const kind = zone.dataset.zone;
  if (kind === 'board') return zone.querySelector<HTMLElement>('[data-card]');
  if (kind === 'field') {
    const isField = zone.matches('input, textarea');
    return isField ? zone : zone.querySelector<HTMLElement>('input, textarea, [contenteditable="true"]');
  }
  if (kind === 'bar') {
    const controls = controlsOf(zone);
    const current = controls.find((c) => c.matches('[aria-pressed="true"][data-pick], [aria-current="page"]'));
    return current ?? controls[0] ?? null;
  }
  const items = itemsOf(zone);
  const item = step > 0 ? items[0] : items[items.length - 1];
  if (!item) return null;
  return kind === 'rows' ? item : (controlsOf(item)[0] ?? null);
}

function focus(el: HTMLElement) {
  el.focus();
  el.scrollIntoView({ block: 'nearest' });
}

/** Focuses the first zone with something to focus, from `from` stepping by `step`. */
function enterFrom(zones: HTMLElement[], from: number, step: number): boolean {
  for (let i = from; i >= 0 && i < zones.length; i += step) {
    const entry = entryOf(zones[i], step);
    if (entry) {
      focus(entry);
      return true;
    }
  }
  return false;
}

/** ← → to the next or previous control in `controls`; a `data-pick` one is pressed. */
function along(controls: HTMLElement[], target: HTMLElement, key: string): boolean {
  const exact = controls.indexOf(target);
  const at = exact >= 0 ? exact : controls.findIndex((c) => c.contains(target));
  if (at < 0) return false;
  const control = controls[key === 'ArrowRight' ? at + 1 : at - 1];
  if (!control) return true;
  focus(control);
  if (control.hasAttribute('data-pick')) control.click();
  return true;
}

/** ↑↓ inside an item zone; false at its first or last item (the key leaves the zone). */
function upDown(zone: HTMLElement, target: HTMLElement, key: string): boolean {
  const items = itemsOf(zone);
  const at = items.findIndex((item) => item === target || item.contains(target));
  const next = key === 'ArrowDown' ? at + 1 : at - 1;
  if (at < 0 || next < 0 || next >= items.length) return false;
  const item = items[next];
  focus(zone.dataset.zone === 'props' ? (controlsOf(item)[0] ?? item) : item);
  return true;
}

/** Whether ↑ / ↓ may leave the field the focus is in. */
function leavesField(target: HTMLElement, key: string): boolean {
  const richEditor = target.isContentEditable || target.closest('.cm-editor') !== null;
  if (richEditor) return false;
  if (target instanceof HTMLTextAreaElement) {
    const atStart = target.selectionStart === 0;
    const atEnd = target.selectionEnd === target.value.length;
    return key === 'ArrowUp' ? atStart : atEnd;
  }
  return true;
}

function handle(node: HTMLElement, e: KeyboardEvent): boolean {
  const target = e.target as HTMLElement;
  const zone = target.closest<HTMLElement>(ZONE);
  if (!zone || !node.contains(zone)) return false;
  const kind = zone.dataset.zone;
  const isVertical = e.key === 'ArrowUp' || e.key === 'ArrowDown';
  const isSideways = e.key === 'ArrowLeft' || e.key === 'ArrowRight';
  const typing = isTypingTarget(target);
  // A field inside a row (renaming it) keeps every key.
  const inRow = target.closest('[data-row]') !== null;
  if (typing && (inRow || isSideways || !leavesField(target, e.key))) return false;
  if (kind === 'rows' && !typing) {
    if (isVertical && upDown(zone, target, e.key)) return true;
    const row = target.closest<HTMLElement>('[data-row]');
    const isFolder = row?.hasAttribute('aria-expanded') ?? false;
    // ← → step through a row's own controls (edit, My day, …); on a folder they fold it.
    if (isSideways && row && !isFolder) return along([row, ...controlsOf(row).filter((c) => c !== row)], target, e.key);
    if (!isVertical) return rowKey(zone, e);
  }
  if (isSideways && kind === 'bar') return along(controlsOf(zone), target, e.key);
  if (isSideways && kind === 'props') {
    const prop = target.closest<HTMLElement>('[data-prop]');
    return prop ? along(controlsOf(prop), target, e.key) : false;
  }
  if (!isVertical) return false;
  if ((kind === 'props' || kind === 'stack') && upDown(zone, target, e.key)) return true;
  const zones = [...node.querySelectorAll<HTMLElement>(ZONE)].filter(isShown);
  const step = e.key === 'ArrowDown' ? 1 : -1;
  return enterFrom(zones, zones.indexOf(zone) + step, step);
}

export function zoneNav(node: HTMLElement) {
  function onkeydown(e: KeyboardEvent) {
    const hasModifier = e.metaKey || e.ctrlKey || e.altKey || e.shiftKey;
    // An open dropdown or dialog owns the keys until it closes.
    if (e.defaultPrevented || hasModifier || hasLayers()) return;
    if (handle(node, e)) e.preventDefault();
  }

  node.addEventListener('keydown', onkeydown);
  return {
    destroy() {
      node.removeEventListener('keydown', onkeydown);
    },
  };
}
