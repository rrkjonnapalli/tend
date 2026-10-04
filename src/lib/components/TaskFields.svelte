<script lang="ts">
  // The task's properties grid: status, due date and time, My day, assignee, tags.
  import type { Task, TaskPatch } from '#lib/types/task.type.ts';
  import StatusSelect from './StatusSelect.svelte';
  import DateField from './DateField.svelte';
  import TimeField from './TimeField.svelte';
  import AssigneeField from './AssigneeField.svelte';
  import TagEditor from './TagEditor.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { inMyDay, isOverdue } from '#lib/util/task-views.ts';

  let {
    task,
    tags,
    tagsLoaded,
    onpatch,
    ontags,
  }: {
    task: Task;
    tags: string[];
    /** Tags come with the task detail; the editor waits for them. */
    tagsLoaded: boolean;
    onpatch: (patch: TaskPatch) => void;
    ontags: (tags: string[]) => void;
  } = $props();

  let statusSelect = $state<StatusSelect>();
  let dueField = $state<DateField>();

  const mine = $derived(inMyDay(task, { today: tasks.day, myDay: tasks.myDay }));

  /** Clearing the date clears the time with it. */
  function setDue(dueDate: string | null) {
    if (dueDate) onpatch({ dueDate });
    else onpatch({ dueDate: null, dueTime: null });
  }

  /** Opens the status list (the "Set status…" command). */
  export function showStatus() {
    statusSelect?.show();
  }

  /** Opens the due date calendar (the "Set due date…" command). */
  export function showDue() {
    dueField?.show();
  }
</script>

<div class="props">
  <span class="pl">Status</span>
  <div class="pv"><StatusSelect bind:this={statusSelect} value={task.status} onchange={(status) => onpatch({ status })} /></div>
  <span class="pl">Due</span>
  <div class="pv">
    <DateField bind:this={dueField} value={task.dueDate} warn={isOverdue(task, tasks.day)} onchange={setDue} />
    <TimeField value={task.dueTime} onchange={(dueTime) => onpatch({ dueTime })} />
  </div>
  <span class="pl">My day</span>
  <div class="pv">
    <button class="toggle" class:add={!mine} class:rem={mine} aria-pressed={mine} onclick={() => tasks.setMyDay(task.id, !mine)}>
      {mine ? '− My day' : '+ My day'}
    </button>
  </div>
  <span class="pl">Assignee</span>
  <div class="pv"><AssigneeField value={task.assigneeId} onchange={(assigneeId) => onpatch({ assigneeId })} /></div>
  <span class="pl">Tags</span>
  <div class="pv">{#if tagsLoaded}<TagEditor {tags} onchange={ontags} addLabel="+" />{/if}</div>
</div>

<style>
  .props {
    display: grid;
    grid-template-columns: 96px minmax(0, 1fr);
    row-gap: 4px;
    align-items: center;
  }

  .pl {
    font: 400 13px/18px var(--font-ui);
    color: var(--c-muted-2);
  }

  .pv {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 32px;
    font: 400 14px/20px var(--font-ui);
  }

  .toggle {
    height: 28px;
    padding: 0 8px;
    margin-left: -8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    font: 400 14px/20px var(--font-ui);
    cursor: pointer;
  }

  .toggle.add {
    color: var(--primary);
  }

  .toggle.add:hover {
    background: var(--primary-soft);
  }

  .toggle.rem {
    color: var(--danger);
  }

  .toggle.rem:hover {
    background: var(--danger-soft);
  }
</style>
