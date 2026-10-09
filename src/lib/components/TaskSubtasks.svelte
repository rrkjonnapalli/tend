<script lang="ts">
  import { rowNav } from '#lib/actions/row-nav.action.ts';
  import AddLine from './AddLine.svelte';
  // A task's subtasks: an add field, the open ones (drag to reorder), then the done ones folded away.
  import type { DroppedOrder } from '#lib/types/reorder.type.ts';
  import type { Task } from '#lib/types/task.type.ts';
  import { onDestroy } from 'svelte';
  import Icon from './Icon.svelte';
  import TaskRow from './TaskRow.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { isDone } from '#lib/util/task-views.ts';
  import { droppedKey, inPreviewOrder, keyAfterLast } from '#lib/util/task-order.ts';
  import { startReorder } from '#lib/util/reorder.ts';

  let { task }: { task: Task } = $props();

  const subs = $derived(tasks.subtasksOf(task.id));
  const openSubs = $derived(subs.filter((s) => !isDone(s)));
  const doneSubs = $derived(subs.filter(isDone));
  let draft = $state('');
  let showDone = $state(false);

  async function add() {
    const title = draft.trim();
    if (!title) return;
    await tasks.create({ path: task.path, title, parentId: task.id, orderKey: keyAfterLast(openSubs) });
    draft = '';
  }

  // ---- reorder

  let preview = $state<string[] | null>(null);
  let lifted = $state<string | null>(null);
  const shown = $derived(inPreviewOrder(openSubs, preview));
  let stopReorder: (() => void) | null = null;

  onDestroy(() => stopReorder?.());

  function showPreview(p: string[] | null) {
    preview = p;
    if (!p) lifted = null;
  }

  function saveDrop(id: string, { order, index }: DroppedOrder) {
    const orderKey = droppedKey({ order, index, tasks: tasks.tasks });
    tasks.update(id, { orderKey });
  }

  function grab(e: PointerEvent, id: string) {
    // The status and My day buttons keep their own clicks.
    if ((e.target as HTMLElement).closest('.mini, [data-no-drag]')) return;
    lifted = id;
    stopReorder = startReorder(e, {
      keys: openSubs.map((s) => s.id),
      key: id,
      preview: showPreview,
      drop: (order, index) => saveDrop(id, { order, index }),
    });
  }
</script>

<div class="nav" use:rowNav>
  <AddLine bind:value={draft} label="Add subtask" placeholder="add subtask" autofocus={false} onadd={add} />
  <div class="subs">
    {#each shown as s (s.id)}
      <TaskRow task={s} compact sortable lifted={preview !== null && lifted === s.id} onpointerdown={(e) => grab(e, s.id)} />
    {/each}
  </div>
  {#if doneSubs.length}
    <button class="donehead" aria-expanded={showDone} onclick={() => (showDone = !showDone)}>
      <Icon name={showDone ? 'chevronDown' : 'chevronRight'} size={10} stroke={3} />Done<span class="cnt">{doneSubs.length}</span>
    </button>
    {#if showDone}
      {#each doneSubs as s (s.id)}
        <TaskRow task={s} compact />
      {/each}
    {/if}
  {/if}
</div>

<style>
  /* Only groups the rows for the keyboard. */
  .nav {
    display: contents;
  }

  .subs {
    display: flex;
    flex-direction: column;
  }

  .donehead {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0;
    margin-top: 4px;
    border: 0;
    background: transparent;
    font: var(--text-caption);
    color: var(--c-muted-2);
    cursor: pointer;
    align-self: flex-start;
  }

  .donehead:hover {
    color: var(--text);
  }

  .cnt {
    margin-left: 2px;
  }
</style>
