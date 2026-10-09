// Order keys and drag previews for task lists.
import type { DroppedOrder } from '#lib/types/reorder.type.ts';
import type { Task } from '#lib/types/task.type.ts';
import { keyAfter, keyBetween } from './order-key.ts';

/** The order key for the task a drag left at `index`, between its new neighbours. */
export function droppedKey({ order, index, tasks }: DroppedOrder & { tasks: Task[] }): string {
  const keyOf = (id: string | undefined) => tasks.find((t) => t.id === id)?.orderKey ?? null;
  const next = order.slice(index + 1).flatMap((id) => keyOf(id) ?? []);
  return keyAfter(keyOf(order[index - 1]), next);
}

/** A key that sorts after every task in `list` (sorted by order key). */
export const keyAfterLast = (list: Task[]) => keyBetween(list.at(-1)?.orderKey ?? null, null);

/** `items` in the order a running drag previews, or as they are when there is no drag. */
export function inPreviewOrder<T extends { id: string }>(items: T[], preview: string[] | null): T[] {
  if (!preview) return items;
  const byId = new Map(items.map((t) => [t.id, t]));
  return preview.map((id) => byId.get(id)).filter((t): t is T => t !== undefined);
}
