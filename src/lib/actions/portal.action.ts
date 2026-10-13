// Moves an element to <body> while it is mounted. `position: fixed` is measured from the nearest
// transformed, filtered or rotated parent, not the window, so anything floating must sit outside them.
// `opener` gives what opened it: focus left inside when it goes returns there, not to nowhere.
export function portal(node: HTMLElement, opener?: () => HTMLElement | null) {
  // Moving an element drops focus from anything inside it (a list that focuses its ticked option).
  const focused = node.contains(document.activeElement) ? (document.activeElement as HTMLElement) : null;
  document.body.appendChild(node);
  focused?.focus();
  return {
    destroy() {
      const hadFocus = node.contains(document.activeElement);
      node.remove();
      if (hadFocus) opener?.()?.focus();
    },
  };
}
