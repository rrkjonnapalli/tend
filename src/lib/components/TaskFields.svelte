<script lang="ts">
  // The task's properties grid: location (top-level tasks), status, due date and time, My day,
  // assignee, tags, labels.
  import type { Task, TaskPatch } from '#lib/types/task.type.ts';
  import StatusSelect from './StatusSelect.svelte';
  import DateField from './DateField.svelte';
  import TimeField from './TimeField.svelte';
  import AssigneeField from './AssigneeField.svelte';
  import TagEditor from './TagEditor.svelte';
  import LabelField from './LabelField.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { inMyDay, isOverdue } from '#lib/util/task-views.ts';
  import { folderLabel } from '#lib/util/task-urls.ts';
  import Icon from './Icon.svelte';

  let {
    task,
    tags,
    tagsLoaded,
    rootName,
    onpatch,
    ontags,
    onmove,
  }: {
    task: Task;
    tags: string[];
    /** Tags come with the task detail; the editor waits for them. */
    tagsLoaded: boolean;
    onpatch: (patch: TaskPatch) => void;
    ontags: (tags: string[]) => void | Promise<void>;
    /** The tasks root folder's display name, for the location. */
    rootName: string;
    /** Opens Move to…; the location button calls it. */
    onmove: () => void;
  } = $props();

  // Subtasks live wherever their parent does, so only top-level tasks show a location.
  const hasLocation = $derived(task.parentId === null);
  const location = $derived(folderLabel({ rootName, path: task.path }));

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

<div class="props" data-zone="props">
  {#if hasLocation}
    <span class="pl">Location</span>
    <div class="pv" data-prop>
      <button class="loc" title="Move to…" onclick={onmove}><Icon name="folder" size={14} />{location}</button>
    </div>
  {/if}
  <span class="pl">Status</span>
  <div class="pv" data-prop><StatusSelect bind:this={statusSelect} value={task.status} onchange={(status) => onpatch({ status })} /></div>
  <span class="pl">Due</span>
  <div class="pv" data-prop>
    <DateField bind:this={dueField} value={task.dueDate} warn={isOverdue(task, tasks.day)} onchange={setDue} />
    <TimeField value={task.dueTime} onchange={(dueTime) => onpatch({ dueTime })} />
  </div>
  <span class="pl">My day</span>
  <div class="pv" data-prop>
    <button class="toggle" class:add={!mine} class:rem={mine} aria-pressed={mine} onclick={() => tasks.setMyDay(task.id, !mine)}>
      {mine ? '− My day' : '+ My day'}
    </button>
  </div>
  <span class="pl">Assignee</span>
  <div class="pv" data-prop><AssigneeField value={task.assigneeId} onchange={(assigneeId) => onpatch({ assigneeId })} /></div>
  <span class="pl">Tags</span>
  <div class="pv" data-prop>{#if tagsLoaded}<TagEditor {tags} onchange={ontags} addLabel="+" />{/if}</div>
  <span class="pl">Labels</span>
  <div class="pv" data-prop><LabelField taskId={task.id} /></div>
</div>

<style>
  .toggle {
    height: 28px;
    padding: 0 8px;
    margin-left: -8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    font: var(--text-ui);
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

  .loc {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 8px;
    margin-left: -8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text);
    font: var(--text-ui);
    cursor: pointer;
  }

  .loc:hover {
    background: var(--c-hover);
  }
</style>
