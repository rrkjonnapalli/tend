// The rules of the phone's cut and paste (tend-mobile-spec.md §8).
import type { TreeItem } from '#lib/types/clip.type.ts';

/** Items can be pasted in `target` unless every one of them is already there. */
export const canPaste = ({ from, target }: { from: string[]; target: string }) => from.some((folder) => folder !== target);

/** The selected items to act on: a subtask whose parent is also selected goes with its parent, so
 *  acting on it again would fail. */
export function selectedRoots(items: TreeItem[]): string[] {
  const selected = new Set(items.map((item) => item.id));
  return items.filter((item) => item.parentId === null || !selected.has(item.parentId)).map((item) => item.id);
}
