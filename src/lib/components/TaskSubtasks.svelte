<script lang="ts">
  // A task's subtasks: an add field, the open ones (drag to reorder), then the done ones folded away.
  import type { DroppedOrder } from '#lib/types/reorder.type.ts';
  import type { Task } from '#lib/types/task.type.ts';
  import { onDestroy } from 'svelte';
  import Icon from './Icon.svelte';
  import Grip from './Grip.svelte';
  import StatusSelect from './StatusSelect.svelte';
  import MyDayButton from './MyDayButton.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { isDone } from '#lib/util/task-views.ts';
  import { droppedKey, inPreviewOrder, keyAfterLast } from '#lib/util/task-order.ts';
  import { startReorder } from '#lib/util/reorder.ts';
  import { routes } from '#lib/util/routes.ts';

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

  function addKeys(e: KeyboardEvent) {
    if (e.key === 'Enter') add();
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

<label class="add">
  <span class="plus">+</span>
  <input placeholder="add subtask" aria-label="Add subtask" bind:value={draft} onkeydown={addKeys} />
</label>
<div class="subs">
  {#each shown as s (s.id)}
    <a
      class="sub sortable reveal-on-hover"
      class:lifted={preview !== null && lifted === s.id}
      href={routes.task(s.id)}
      draggable="false"
      data-key={s.id}
      onpointerdown={(e) => grab(e, s.id)}>
      <Grip />
      <StatusSelect compact value={s.status} onchange={(status) => tasks.update(s.id, { status })} />
      <span class="st">{s.title}</span>
      <MyDayButton task={s} />
    </a>
  {/each}
</div>
{#if doneSubs.length}
  <button class="donehead" aria-expanded={showDone} onclick={() => (showDone = !showDone)}>
    <Icon name={showDone ? 'chevronDown' : 'chevronRight'} size={10} stroke={3} />Done<span class="cnt">{doneSubs.length}</span>
  </button>
  {#if showDone}
    {#each doneSubs as s (s.id)}
      <a class="sub reveal-on-hover" href={routes.task(s.id)}>
        <StatusSelect compact value={s.status} onchange={(status) => tasks.update(s.id, { status })} />
        <span class="st done">{s.title}</span>
        <MyDayButton task={s} />
      </a>
    {/each}
  {/if}
{/if}

<style>
  .add {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 32px;
    font: 400 14px/20px var(--font-ui);
    color: var(--c-muted-2);
  }

  .plus {
    width: 8px;
    text-align: center;
  }

  .add input {
    flex: 1;
    border: 0;
    outline: none;
    background: transparent;
    font: 400 14px/20px var(--font-ui);
    color: var(--text);
  }

  .add input::placeholder {
    color: var(--c-muted-2);
  }

  .subs {
    display: flex;
    flex-direction: column;
  }

  .sub {
    position: relative;
    display: flex;
    align-items: center;
    gap: 12px;
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

  .st {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .done {
    color: var(--c-muted-2);
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
    font: 400 13px/18px var(--font-ui);
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
