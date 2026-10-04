<script lang="ts">
  // One subtask on its parent's page: grip (open ones) · status dot · title · edit · My day.
  // The row links to the subtask; while its title is edited in place it stops being a link.
  import type { Task } from '#lib/types/task.type.ts';
  import Grip from './Grip.svelte';
  import StatusSelect from './StatusSelect.svelte';
  import MyDayButton from './MyDayButton.svelte';
  import EditButton from './EditButton.svelte';
  import InlineTitle from './InlineTitle.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { isDone } from '#lib/util/task-views.ts';
  import { routes } from '#lib/util/routes.ts';

  let {
    sub,
    sortable = false,
    lifted = false,
    onpointerdown,
  }: {
    sub: Task;
    /** Open subtasks can be dragged to reorder. */
    sortable?: boolean;
    lifted?: boolean;
    onpointerdown?: (e: PointerEvent) => void;
  } = $props();

  let editing = $state(false);

  const titleProblem = (text: string) => (text ? '' : 'A subtask needs a title');

  function saveTitle(title: string) {
    editing = false;
    tasks.update(sub.id, { title });
  }

  function setStatus(status: string) {
    tasks.update(sub.id, { status });
  }
</script>

{#if editing}
  <div class="sub editing" data-key={sub.id}>
    {#if sortable}<Grip />{/if}
    <StatusSelect compact value={sub.status} onchange={setStatus} />
    <InlineTitle value={sub.title} label="Subtask title" check={titleProblem} onsave={saveTitle} oncancel={() => (editing = false)} />
    <span class="md"><MyDayButton task={sub} /></span>
  </div>
{:else}
  <a class="sub reveal-on-hover" class:sortable class:lifted href={routes.task(sub.id)} draggable="false" data-key={sub.id} {onpointerdown}>
    {#if sortable}<Grip />{/if}
    <StatusSelect compact value={sub.status} onchange={setStatus} />
    <span class="st" class:done={isDone(sub)}>{sub.title}</span>
    <EditButton label="Edit title" onedit={() => (editing = true)} />
    <span class="md"><MyDayButton task={sub} /></span>
  </a>
{/if}

<style>
  .sub {
    position: relative;
    display: flex;
    align-items: center;
    gap: var(--col-gap);
    min-height: 32px;
    padding: 0 16px 0 32px;
    margin: 0 -16px 0 -32px;
    font: 400 14px/20px var(--font-ui);
    color: var(--text);
    text-decoration: none;
    user-select: none;
    touch-action: pan-x;
  }

  .sub:hover {
    background: var(--c-hover);
    color: var(--text);
  }

  .sub.editing:hover {
    background: transparent;
  }

  /* A fixed slot, so the edit pencil sits at the same spot with or without "+ my day". */
  .md {
    flex: 0 0 var(--col-myday);
    display: flex;
  }

  .st {
    flex: 1;
    margin-left: 6px;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .done {
    color: var(--c-muted-2);
  }
</style>
