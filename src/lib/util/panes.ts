// ⌘← ⌘→ ⌘↓ ⌘↑: between the side pane (the explorer, the Settings sections), the page's main
// content and the dock. The side pane is `data-pane="side"`, the dock `data-pane="dock"`, the
// main content the page's <main>.

/** Where focus lands, best first (a selector list would match in page order instead). */
const SIDE_ENTRY = ['[data-row][aria-selected="true"]', '[aria-current="page"]', '[data-row]', 'button'];
const DOCK_ENTRY = ['[aria-current="page"]', 'a[href]', 'button'];
const MAIN_ENTRY = ['[data-zone="rows"] [data-row]', '[data-zone="board"] [data-card]', '[data-zone] input', 'button', 'a[href]'];

function firstOf(root: ParentNode, selectors: string[]): HTMLElement | null {
  for (const selector of selectors) {
    const found = root.querySelector<HTMLElement>(selector);
    if (found) return found;
  }
  return null;
}

/** Focuses the open item in the side pane (or its first row); false when there is no side pane. */
export function focusSide(root: ParentNode): boolean {
  const side = root.querySelector<HTMLElement>('[data-pane="side"]');
  const entry = side ? firstOf(side, SIDE_ENTRY) : null;
  entry?.focus();
  return Boolean(entry);
}

/** Focuses the dock on the current section's icon; false when the dock isn't shown. */
export function focusDock(root: ParentNode): boolean {
  const dock = root.querySelector<HTMLElement>('[data-pane="dock"]');
  const entry = dock ? firstOf(dock, DOCK_ENTRY) : null;
  entry?.focus();
  return Boolean(entry);
}

/** Focuses where the user last was in the main content, or its first list row or control. */
export function focusMain(root: ParentNode, last: HTMLElement | null): boolean {
  const main = root.querySelector<HTMLElement>('main');
  if (!main) return false;
  const lastStillThere = last !== null && last.isConnected && main.contains(last);
  const entry = lastStillThere ? last : firstOf(main, MAIN_ENTRY);
  entry?.focus();
  return Boolean(entry);
}
