<script lang="ts">
  // Settings → Tasks → Labels: the fixed list tasks pick their labels from. Deleting a label in use
  // asks first, then takes it off those tasks.
  import ColorListEditor from './ColorListEditor.svelte';
  import { labelsAPI } from '#lib/api/labels.ts';
  import type { ColorItem, ColorListOps } from '#lib/types/color-list.type.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { plural } from '#lib/util/plural.ts';

  tasks.loadLabels();

  const items = $derived(
    tasks.labels.map((l): ColorItem => ({ key: l.id, name: l.name, color: l.color, meta: l.id })),
  );

  const intro =
    'Labels mark tasks beyond their status, like “pending approval” or “script”. A task can carry several; it picks them from this list.';

  async function removeWarning(id: string): Promise<string | null> {
    const uses = await labelsAPI.usage(id);
    if (uses === 0) return null;
    return `It comes off ${plural(uses, 'task')} that carry it.`;
  }

  const ops: ColorListOps = {
    add: (entry) => labelsAPI.add(entry),
    rename: ({ key, name }) => labelsAPI.rename({ id: key, name }),
    recolor: ({ key, color }) => labelsAPI.recolor({ id: key, color }),
    reorder: (order) => labelsAPI.reorder(order),
    remove: (id) => labelsAPI.remove(id),
    // The tasks too: a deleted label comes off them.
    reload: () => tasks.load(),
    removeWarning,
  };
</script>

<ColorListEditor title="Labels" {intro} what="label" {items} {ops} />
