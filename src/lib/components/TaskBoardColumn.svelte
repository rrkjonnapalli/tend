<script lang="ts">
  import { plainText } from '#lib/actions/plain-text.action.ts';
  // One board column: a status header, its cards, and the inline "+ add task" field.
  import type { Status } from '#lib/types/status.type.ts';
  import type { Task } from '#lib/types/task.type.ts';
  import { focusOnMount } from '#lib/actions/focus.action.ts';
  import StatusDot from './StatusDot.svelte';
  import TaskBoardCard from './TaskBoardCard.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { keyAfterLast } from '#lib/util/task-order.ts';

  let {
    status,
    cards,
    folder,
    liftedId,
    adding,
    onstartadd,
    onstopadd,
    ongrab,
  }: {
    status: Status;
    /** The column's cards in order, the dragged one included where it would land. */
    cards: Task[];
    folder: string;
    liftedId: string | null;
    /** This column's add field is open (one column at a time). */
    adding: boolean;
    onstartadd: () => void;
    onstopadd: () => void;
    ongrab: (e: PointerEvent, task: Task) => void;
  } = $props();

  let draft = $state('');

  function startAdding() {
    draft = '';
    onstartadd();
  }

  function stopAdding() {
    draft = '';
    onstopadd();
  }

  async function add() {
    const title = draft.trim();
    if (!title) return;
    await tasks.create({ path: folder, title, status: status.value, orderKey: keyAfterLast(cards) });
    draft = '';
  }

  function addKeys(e: KeyboardEvent) {
    if (e.key === 'Enter') add();
    if (e.key === 'Escape') stopAdding();
  }

  /** Leaving an empty field closes it; a typed title stays. */
  function onAddBlur() {
    if (!draft.trim()) onstopadd();
  }
</script>

<section class="col" data-col={status.value} aria-label={status.label}>
  <header class="colh"><StatusDot value={status.value} /><span>{status.label}</span><span class="vc">{cards.length || ''}</span></header>
  <div class="cards">
    {#each cards as t (t.id)}
      <TaskBoardCard task={t} {folder} lifted={liftedId === t.id} onpointerdown={(e) => ongrab(e, t)} />
    {/each}
    {#if cards.length === 0 && !adding}<span class="cempty">No tasks</span>{/if}
    {#if adding}
      <input
        use:plainText
        class="cadd"
        aria-label="New task in {status.label}"
        placeholder="Task title, ↵ to add"
        bind:value={draft}
        use:focusOnMount
        onkeydown={addKeys}
        onblur={onAddBlur} />
    {:else}
      <button class="caddb" onclick={startAdding}>+ add task</button>
    {/if}
  </div>
</section>

<style>
  .col {
    flex: 1 1 0;
    min-width: max(200px, calc((100% - (var(--board-fit) - 1) * var(--board-gap)) / var(--board-fit)));
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 8px 8px;
    background: var(--c-hover);
    border-radius: var(--radius-md);
  }

  .colh {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 4px;
    font: 500 13px/18px var(--font-heading);
  }

  .vc {
    font: var(--text-meta);
    color: var(--c-muted-2);
  }

  .cards {
    flex: 1 1 auto;
    min-height: 44px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .cadd {
    width: 100%;
    height: 36px;
    padding: 0 10px;
    border: 1px solid var(--primary);
    border-radius: var(--radius-control);
    outline: none;
    background: var(--c-page);
    font: var(--text-ui);
  }

  .cadd::placeholder {
    color: var(--c-faint);
  }

  .caddb {
    height: 30px;
    padding: 0 6px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    text-align: left;
    color: var(--c-muted-2);
    font: var(--text-caption);
    cursor: pointer;
  }

  .caddb:hover {
    color: var(--text);
    background: var(--c-selected);
  }

  .cempty {
    padding: 12px 4px;
    font: var(--text-meta);
    color: var(--c-faint);
    text-align: center;
  }
</style>
