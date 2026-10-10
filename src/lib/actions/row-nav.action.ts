// The keyboard cursor in a list (notes, tasks, people, the explorer tree). Rows opt in with
// `data-row`; inside a row, `data-open` is what Enter (or Space) presses (else the row itself), `data-rename`
// what F2 presses, and `data-toggle` what → and ← press on a folder (`aria-expanded`).
// ↓ from a field marked `data-row-entry` (the search and add fields above the list) goes to the
// first row.
import { moveTo } from '#lib/util/list-nav.ts';
import { isTypingTarget } from '#lib/util/keys.ts';
import { hasLayers } from '#lib/util/layers.ts';

const ROW = '[data-row]';

function press(row: HTMLElement, selector: string): boolean {
  const control = row.querySelector<HTMLElement>(selector);
  control?.click();
  return control !== null;
}

/** Enter: the row's open control, or the row itself (a link, or a row that opens on click). */
function open(row: HTMLElement) {
  if (!press(row, '[data-open]')) row.click();
}

/** → opens a closed folder, ← closes an open one. */
function fold(row: HTMLElement, key: string) {
  const state = row.getAttribute('aria-expanded');
  const wantsOpen = key === 'ArrowRight' && state === 'false';
  const wantsClose = key === 'ArrowLeft' && state === 'true';
  if (wantsOpen || wantsClose) press(row, '[data-toggle]');
}

function focusRow(row: HTMLElement) {
  row.focus();
  row.scrollIntoView({ block: 'nearest' });
}

/** Handles a key for the rows inside `node`; true when it did something. */
export function rowKey(node: HTMLElement, e: KeyboardEvent): boolean {
  const hasModifier = e.metaKey || e.ctrlKey || e.altKey || e.shiftKey;
  // An open dropdown or dialog owns the keys until it closes.
  if (e.defaultPrevented || hasModifier || hasLayers()) return false;
  const target = e.target as HTMLElement;
  const row = target.closest<HTMLElement>(ROW);
  const typing = isTypingTarget(target);
  // Keys in a field belong to it, except ↓ from an entry field.
  const isEntry = target.closest('[data-row-entry]') !== null;
  if (typing && !isEntry) return false;
  const rows = [...node.querySelectorAll<HTMLElement>(ROW)];
  const at = row ? rows.indexOf(row) : -1;
  const next = moveTo({ key: e.key, at, count: rows.length });
  if (next !== null) {
    e.preventDefault();
    focusRow(rows[next]);
    return true;
  }
  if (!row || row !== target) return false;
  // Enter or Space opens, like pressing a button.
  if (e.key === 'Enter' || e.key === ' ') {
    e.preventDefault();
    open(row);
    return true;
  }
  if (e.key === 'F2') {
    const renamed = press(row, '[data-rename]');
    if (renamed) e.preventDefault();
    return renamed;
  }
  if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') {
    e.preventDefault();
    fold(row, e.key);
    return true;
  }
  return false;
}

export function rowNav(node: HTMLElement) {
  const onkeydown = (e: KeyboardEvent) => rowKey(node, e);
  node.addEventListener('keydown', onkeydown);
  return {
    destroy() {
      node.removeEventListener('keydown', onkeydown);
    },
  };
}
