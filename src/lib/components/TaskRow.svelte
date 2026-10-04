<script lang="ts">
  // One task in a list: status dot · title (subtasks marked) · edit · subtasks · path · due · My day.
  import type { Task } from '#lib/types/task.type.ts';
  import StatusSelect from './StatusSelect.svelte';
  import Grip from './Grip.svelte';
  import SubtaskMark from './SubtaskMark.svelte';
  import EditButton from './EditButton.svelte';
  import InlineTitle from './InlineTitle.svelte';
  import MyDayButton from './MyDayButton.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { dueLabel } from '#lib/util/dates.ts';
  import { isDone, isOverdue } from '#lib/util/task-views.ts';
  import { relativeTo } from '#lib/util/names.ts';
  import { routes } from '#lib/util/routes.ts';
  import { goto } from '$app/navigation';

  let {
    task,
    folder,
    sortable = false,
    lifted = false,
    onpointerdown,
  }: {
    task: Task;
    /** Paths are shown relative to this folder. */
    folder: string;
    sortable?: boolean;
    lifted?: boolean;
    onpointerdown?: (e: PointerEvent) => void;
  } = $props();

  const now = $derived(tasks.day);

  let editing = $state(false);

  const titleProblem = (text: string) => (text ? '' : 'A task needs a title');

  function saveTitle(title: string) {
    editing = false;
    tasks.update(task.id, { title });
  }
</script>

<div class="row reveal-on-hover" class:editing class:sortable class:lifted data-key={task.id} role="listitem" {onpointerdown}>
  {#if sortable}<Grip />{/if}
  <StatusSelect compact value={task.status} onchange={(status) => tasks.update(task.id, { status })} />
  {#if editing}
    <InlineTitle value={task.title} label="Task title" check={titleProblem} onsave={saveTitle} oncancel={() => (editing = false)} />
  {:else}
    <button class="title" onclick={() => goto(routes.task(task.id))}>
      <SubtaskMark {task} /><span class:done={isDone(task)}>{task.title}</span>
    </button>
  {/if}
  <span class="ed">{#if !editing}<EditButton label="Edit title" onedit={() => (editing = true)} />{/if}</span>
  <span class="meta">{tasks.subtaskCount(task.id)}</span>
  <span class="meta path">{relativeTo(task.path, folder)}</span>
  <span class="due" class:warn={isOverdue(task, now)}>{task.dueDate ? dueLabel(task.dueDate) : ''}</span>
  <span class="md">
    <MyDayButton {task} />
  </span>
</div>

<style>
  .row {
    position: relative;
    display: grid;
    /* Fixed right-hand columns, so the edit pencil lines up on every row. */
    grid-template-columns: 8px minmax(0, 1fr) 24px 36px 160px 72px 64px;
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

  .row.editing:hover {
    background: transparent;
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

  .ed {
    display: grid;
  }

  .done {
    color: var(--c-muted-2);
  }

  .meta {
    font: 400 13px/18px var(--font-ui);
    color: var(--c-muted-2);
  }

  .path {
    text-align: right;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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

</style>
