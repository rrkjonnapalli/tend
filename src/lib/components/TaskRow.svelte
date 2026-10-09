<script lang="ts">
  // One task in a list. Left: status dot · type badge · title. Right, each a fixed width so they
  // line up on every row: edit · My day · subtask count · due · assignee · path (cut off, tooltip).
  // `compact` is the subtask list on a task page: edit · My day · assignee only, shorter rows, and
  // no subtask badge (every row there is one).
  import type { Task } from '#lib/types/task.type.ts';
  import StatusSelect from './StatusSelect.svelte';
  import Grip from './Grip.svelte';
  import SubtaskMark from './SubtaskMark.svelte';
  import EditButton from './EditButton.svelte';
  import InlineTitle from './InlineTitle.svelte';
  import AssigneeBadge from './AssigneeBadge.svelte';
  import { overflowTitle } from '#lib/actions/overflow-title.action.ts';
  import MyDayButton from './MyDayButton.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { dueLabel } from '#lib/util/dates.ts';
  import { isDone, isOverdue } from '#lib/util/task-views.ts';
  import { relativeTo } from '#lib/util/names.ts';
  import { routes } from '#lib/util/routes.ts';
  import { goto } from '$app/navigation';

  let {
    task,
    folder = '',
    compact = false,
    sortable = false,
    lifted = false,
    onpointerdown,
  }: {
    task: Task;
    /** Paths are shown relative to this folder. */
    folder?: string;
    compact?: boolean;
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

<div class="row reveal-on-hover" class:compact class:editing class:sortable class:lifted data-key={task.id} role="listitem" {onpointerdown}>
  {#if sortable}<Grip />{/if}
  <StatusSelect compact value={task.status} onchange={(status) => tasks.update(task.id, { status })} />
  {#if editing}
    <InlineTitle value={task.title} label="Task title" check={titleProblem} onsave={saveTitle} oncancel={() => (editing = false)} />
  {:else}
    <button class="title" onclick={() => goto(routes.task(task.id))}>
      {#if !compact}<SubtaskMark {task} />{/if}<span class:done={isDone(task)}>{task.title}</span>
    </button>
  {/if}
  <span class="ed">{#if !editing}<EditButton label="Edit title" onedit={() => (editing = true)} />{/if}</span>
  <span class="md"><MyDayButton {task} /></span>
  {#if !compact}
    <span class="meta count">{tasks.subtaskCount(task.id)}</span>
    <span class="due" class:warn={isOverdue(task, now)}>{task.dueDate ? dueLabel(task.dueDate) : ''}</span>
  {/if}
  <span class="who"><AssigneeBadge value={task.assigneeId} onchange={(assigneeId) => tasks.update(task.id, { assigneeId })} /></span>
  {#if !compact}<span class="meta path" use:overflowTitle>{relativeTo(task.path, folder)}</span>{/if}
</div>

<style>
  .row {
    position: relative;
    display: grid;
    /* dot · title | edit · My day · count · due · assignee · path */
    grid-template-columns: var(--grid-task);
    align-items: center;
    column-gap: var(--col-gap);
    min-height: 36px;
    padding: 0 16px;
    margin: 0 -16px;
    font: var(--text-ui);
  }

  .row.compact {
    grid-template-columns: var(--grid-subtask);
    min-height: 32px;
  }

  .compact .title {
    height: 32px;
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
    font: var(--text-caption);
    color: var(--c-muted-2);
  }

  .path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .due {
    font: var(--text-caption);
    color: var(--c-muted-2);
  }

  .due.warn {
    color: var(--c-overdue);
  }

  .md,
  .who {
    display: flex;
  }

</style>
