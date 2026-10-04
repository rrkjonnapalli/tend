// Open popovers and dialogs, newest last. Esc closes only the newest one.

type Layer = { close: () => void };

const stack: Layer[] = [];

function onKey(e: KeyboardEvent) {
  if (e.key !== 'Escape' || stack.length === 0) return;
  e.preventDefault();
  e.stopPropagation();
  stack[stack.length - 1].close();
}

if (typeof window !== 'undefined') window.addEventListener('keydown', onKey, true);

/** Registers a layer; returns the function that removes it. */
export function pushLayer(close: () => void): () => void {
  const layer = { close };
  stack.push(layer);
  return () => {
    const i = stack.indexOf(layer);
    if (i >= 0) stack.splice(i, 1);
  };
}

export const hasLayers = () => stack.length > 0;
