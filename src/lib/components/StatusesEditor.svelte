<script lang="ts">
  // Settings → Tasks → Statuses: the board's columns. The first and last stay put; a default status
  // can't be deleted, and the server refuses one still in use.
  import ColorListEditor from './ColorListEditor.svelte';
  import { statusesAPI } from '#lib/api/statuses.ts';
  import type { ColorItem, ColorListOps } from '#lib/types/color-list.type.ts';
  import type { StatusPosition } from '#lib/types/status.type.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { FIRST_STATUS } from '#lib/util/statuses.ts';

  tasks.loadStatuses();

  const PIN_NOTES: Record<StatusPosition, string> = { first: 'always first', middle: '', last: 'always last' };

  function positionOf(index: number, count: number): StatusPosition {
    if (index === 0) return 'first';
    if (index === count - 1) return 'last';
    return 'middle';
  }

  const items = $derived(
    tasks.statuses.map((s, i): ColorItem => {
      const position = positionOf(i, tasks.statuses.length);
      return {
        key: s.value,
        name: s.label,
        color: s.color,
        meta: s.value,
        ring: s.value === FIRST_STATUS,
        pinned: position !== 'middle',
        note: PIN_NOTES[position],
        removable: !s.isDefault,
      };
    }),
  );

  const first = $derived(tasks.statuses[0]?.label ?? 'Not started');
  const last = $derived(tasks.statuses.at(-1)?.label ?? 'Done');
  const intro = $derived(
    `Each status is a column on the board; its color is used for the status dot everywhere. ${first} is always first and ${last} always last; new tasks start as ${first}.`,
  );

  const ops: ColorListOps = {
    add: ({ name, value, color }) => statusesAPI.add({ label: name, value, color }),
    rename: ({ key, name }) => statusesAPI.rename({ value: key, label: name }),
    recolor: ({ key, color }) => statusesAPI.recolor({ value: key, color }),
    reorder: (order) => statusesAPI.reorder(order),
    remove: (key) => statusesAPI.remove(key),
    reload: () => tasks.loadStatuses(),
  };
</script>

<ColorListEditor title="Statuses" {intro} what="status" {items} {ops} pinEnds />
