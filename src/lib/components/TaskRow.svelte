<script lang="ts">
  // One task in a list: status dot · title (with parent hint) · subtasks · path · due · My day.
  import type { Task } from '#lib/api/tasks.ts';
  import StatusDot from './StatusDot.svelte';
  import Grip from './Grip.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { dueLabel } from '#lib/util/dates.ts';
  import { inMyDay, isDone, isOverdue } from '#lib/util/task-views.ts';
  import { relativeTo } from '#lib/util/names.ts';
  import { routes } from '#lib/util/routes.ts';
  import { goto } from '$app/navigation';

  let {
    task,
    folder,
    showParent = false,
    sortable = false,
    lifted = false,
    onpointerdown,
  }: {
    task: Task;
    /** Paths are shown relative to this folder. */
    folder: string;
    showParent?: boolean;
    sortable?: boolean;
    lifted?: boolean;
    onpointerdown?: (e: PointerEvent) => void;
  } = $props();

  const now = $derived(tasks.day);
  const parent = $derived(showParent && task.parentId ? tasks.byId(task.parentId) : undefined);
  const mine = $derived(inMyDay(task, now, tasks.myDay));
</script>

<div class="row reveal-on-hover" class:sortable class:lifted data-key={task.id} role="listitem" {onpointerdown}>
  {#if sortable}<Grip />{/if}
  <StatusDot value={task.status} />
  <button class="title" onclick={() => goto(routes.task(task.id))}>
    {#if parent}<span class="meta">{parent.title} › </span>{/if}<span class:done={isDone(task)}>{task.title}</span>
  </button>
  <span class="meta">{tasks.subtaskCount(task.id)}</span>
  <span class="meta">{relativeTo(task.path, folder)}</span>
  <span class="due" class:warn={isOverdue(task, now)}>{task.dueDate ? dueLabel(task.dueDate) : ''}</span>
  <span class="md">
    {#if mine}
      <button class="mini rem on-hover" onclick={() => tasks.setMyDay(task.id, false)}>− my day</button>
    {:else if !isDone(task)}
      <button class="mini add on-hover" onclick={() => tasks.setMyDay(task.id, true)}>+ my day</button>
    {/if}
  </span>
</div>

<style>
  .row {
    position: relative;
    display: grid;
    grid-template-columns: 8px minmax(0, 1fr) auto auto 72px 64px;
    align-items: center;
    gap: 12px;
    min-height: 36px;
    padding: 0 16px;
    margin: 0 -16px;
    font: 400 14px/20px var(--font-ui);
  }

  .row.sortable {
    padding-left: 32px;
    margin-left: -32px;
    touch-action: pan-x;
    user-select: none;
  }

  .row:hover {
    background: var(--c-hover);
  }

  :global(body.reordering) .row:not(.lifted):hover {
    background: transparent;
  }

  .title {
    min-width: 0;
    height: 36px;
    padding: 0;
    border: 0;
    background: transparent;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: inherit;
    color: inherit;
    cursor: pointer;
  }

  .done {
    color: var(--c-muted-2);
  }

  .meta {
    font: 400 13px/18px var(--font-ui);
    color: var(--c-muted-2);
  }

  .due {
    font: 400 13px/18px var(--font-ui);
    color: var(--c-muted-2);
    text-align: right;
  }

  .due.warn {
    color: var(--c-overdue);
  }

  .md {
    text-align: right;
  }

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

  .mini.add {
    color: var(--primary);
  }

  .mini.add:hover {
    background: var(--primary-soft);
  }

  .mini.rem {
    color: var(--danger);
  }

  .mini.rem:hover {
    background: var(--danger-soft);
  }
</style>
