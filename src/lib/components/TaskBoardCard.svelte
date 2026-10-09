<script lang="ts">
  // One board card: title, then due · subtasks · labels (dots) · path · assignee. Opens the task on
  // click or Enter.
  import type { Task } from '#lib/types/task.type.ts';
  import { goto } from '$app/navigation';
  import Avatar from './Avatar.svelte';
  import SubtaskMark from './SubtaskMark.svelte';
  import LabelDots from './LabelDots.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { people } from '#lib/stores/people.svelte.ts';
  import { dueLabel } from '#lib/util/dates.ts';
  import { isDone, isOverdue } from '#lib/util/task-views.ts';
  import { relativeTo } from '#lib/util/names.ts';
  import { routes } from '#lib/util/routes.ts';

  let {
    task,
    folder,
    lifted,
    onpointerdown,
  }: {
    task: Task;
    /** Paths are shown relative to this folder. */
    folder: string;
    /** This card is being dragged: draw the dashed placeholder. */
    lifted: boolean;
    onpointerdown: (e: PointerEvent) => void;
  } = $props();

  const who = $derived(people.byId(task.assigneeId));
  const rel = $derived(relativeTo(task.path, folder));
  const subs = $derived(tasks.subtaskCount(task.id));
  const hasLabels = $derived(tasks.labelsOf(task.id).length > 0);
  const hasMeta = $derived(!!task.dueDate || !!subs || hasLabels || !!rel || !!who);

  function open() {
    goto(routes.task(task.id));
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') open();
  }
</script>

<div class="card" class:lifted class:isdone={isDone(task)} data-card={task.id} role="button" tabindex="0" {onpointerdown} onclick={open} {onkeydown}>
  <span class="ctitle"><SubtaskMark {task} />{task.title}</span>
  {#if hasMeta}
    <span class="cmeta">
      {#if task.dueDate}<span class:warn={isOverdue(task, tasks.day)}>{dueLabel(task.dueDate)}</span>{/if}
      {#if subs}<span>{subs}</span>{/if}
      <LabelDots taskId={task.id} />
      {#if rel}<span>{rel}</span>{/if}
      {#if who}<span class="cav"><Avatar person={who} size={18} /></span>{/if}
    </span>
  {/if}
</div>

<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px 10px;
    background: var(--c-page);
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    font: var(--text-ui);
    cursor: pointer;
    touch-action: none;
  }

  .card:hover {
    border-color: var(--c-faint);
  }

  .card.lifted {
    background: transparent;
    border: 1px dashed var(--accent);
  }

  .card.lifted > * {
    visibility: hidden;
  }

  :global(.card.ghost) {
    border-color: var(--accent) !important;
    box-shadow: inset 2px 0 0 var(--accent);
    opacity: 0.96;
    will-change: transform;
  }

  .isdone .ctitle {
    color: var(--c-muted-2);
  }

  .cmeta {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    font: var(--text-meta);
    color: var(--c-muted-2);
  }

  .warn {
    color: var(--c-overdue);
  }

  .cav {
    display: grid;
    margin-left: auto;
  }
</style>
