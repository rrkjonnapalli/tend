// Pointer drag to reorder a vertical list of equal-height rows. While dragging, `preview`
// receives the keys in their new order; on release `drop` receives the final order (only when
// it changed). Rows can only move within [min, max] (e.g. open tasks stay above done ones).

export type ReorderOptions = {
  keys: string[];
  key: string;
  min?: number;
  max?: number;
  preview: (keys: string[] | null) => void;
  drop: (keys: string[], index: number) => void;
};

const START_PX = 4;

export function startReorder(e: PointerEvent, o: ReorderOptions) {
  if (e.button !== 0) return;
  // The handle may sit inside the row; the row is the nearest element with data-key.
  const row = (e.currentTarget as HTMLElement).closest<HTMLElement>('[data-key]');
  const first = row?.parentElement?.querySelector('[data-key]');
  if (!row || !first) return;
  const top = first.getBoundingClientRect().top;
  const height = row.getBoundingClientRect().height || 36;
  const y0 = e.clientY;
  const [min, max] = [o.min ?? 0, o.max ?? o.keys.length - 1];
  let order: string[] | null = null;

  const move = (ev: PointerEvent) => {
    if (!order) {
      if (Math.abs(ev.clientY - y0) < START_PX) return;
      order = o.keys.slice();
      document.body.classList.add('reordering');
    }
    const idx = Math.max(min, Math.min(max, Math.floor((ev.clientY - top) / height)));
    const next = o.keys.filter((k) => k !== o.key);
    next.splice(idx, 0, o.key);
    if (next.join('|') !== order.join('|')) {
      order = next;
      o.preview(next);
    }
  };

  const up = () => {
    window.removeEventListener('pointermove', move);
    window.removeEventListener('pointerup', up);
    window.removeEventListener('pointercancel', up);
    document.body.classList.remove('reordering');
    if (!order) return;
    // Swallow the click that ends a drag.
    window.addEventListener('click', (c) => c.stopPropagation(), { capture: true, once: true });
    o.preview(null);
    if (order.join('|') !== o.keys.join('|')) o.drop(order, order.indexOf(o.key));
  };

  window.addEventListener('pointermove', move);
  window.addEventListener('pointerup', up);
  window.addEventListener('pointercancel', up);
}
