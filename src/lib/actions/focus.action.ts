// Svelte actions for fields that should take focus as soon as they appear.

export function focusOnMount(node: HTMLElement) {
  node.focus();
}

/** Focuses the field on mount only when `enabled`, e.g. a page opened for a new record. */
export function focusWhen(node: HTMLElement, enabled: boolean) {
  if (enabled) node.focus();
}

/** Focuses the field on mount when `enabled` and nothing else has focus: a page that opens lands
 *  in it, but a field that reappears (a task view with an add line) doesn't take the keyboard
 *  from where the user is. */
export function focusIfIdle(node: HTMLElement, enabled: boolean) {
  const idle = document.activeElement === null || document.activeElement === document.body;
  if (enabled && idle) node.focus();
}

/** Focuses an input and selects its text, so typing replaces it. */
export function focusAndSelect(node: HTMLInputElement | HTMLTextAreaElement) {
  node.focus();
  node.select();
}
