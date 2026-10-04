<script lang="ts">
  // "+ my day" (blue) or "− my day" (red), shown while its row is hovered.
  import type { Task } from '#lib/types/task.type.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { inMyDay, isDone } from '#lib/util/task-views.ts';

  let { task }: { task: Task } = $props();

  const mine = $derived(inMyDay(task, tasks.day, tasks.myDay));

  // The button may sit inside a link row: don't follow the link.
  function toggle(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    tasks.setMyDay(task.id, !mine);
  }
</script>

{#if mine}
  <button class="mini rem on-hover" onclick={toggle}>− my day</button>
{:else if !isDone(task)}
  <button class="mini add on-hover" onclick={toggle}>+ my day</button>
{/if}

<style>
  .mini {
    height: 24px;
    padding: 0 6px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    font: 400 12px/16px var(--font-ui);
    white-space: nowrap;
    cursor: pointer;
  }

  .add {
    color: var(--primary);
  }

  .add:hover {
    background: var(--primary-soft);
  }

  .rem {
    color: var(--danger);
  }

  .rem:hover {
    background: var(--danger-soft);
  }
</style>
