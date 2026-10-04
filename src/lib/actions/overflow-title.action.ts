// Shows the full text as a tooltip, but only while it is cut off by an ellipsis.
export function overflowTitle(node: HTMLElement) {
  function update() {
    const cut = node.scrollWidth > node.clientWidth;
    if (cut) node.title = node.textContent ?? '';
    else node.removeAttribute('title');
  }
  node.addEventListener('mouseenter', update);
  return {
    destroy() {
      node.removeEventListener('mouseenter', update);
    },
  };
}
