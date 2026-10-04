// Svelte actions for fields that should take focus as soon as they appear.

export function focusOnMount(node: HTMLElement) {
  node.focus();
}

/** Focuses an input and selects its text, so typing replaces it. */
export function focusAndSelect(node: HTMLInputElement | HTMLTextAreaElement) {
  node.focus();
  node.select();
}
