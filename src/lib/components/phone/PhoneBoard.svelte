<script lang="ts">
  // The task board on a phone (MD-Board): one status column at a time with the next one peeking;
  // swipe between them (scroll snapping); dots show which column is in view. Cards are soft fills.
  // Each column ends with "add task", which adds one with that column's status.
  import type { Task } from '#lib/types/task.type.ts';
  import StatusDot from '../StatusDot.svelte';
  import Icon from '../Icon.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { isOverdue } from '#lib/util/task-views.ts';
  import { dueLabel } from '#lib/util/dates.ts';
  import { routes } from '#lib/util/routes.ts';

  let { tasks: shown, folder }: { tasks: Task[]; folder: string } = $props();

  let strip = $state<HTMLDivElement>();
  let current = $state(0);
  let addingTo = $state<string | null>(null);
  let draft = $state('');

  const columns = $derived([...tasks.statuses].sort((a, b) => a.position - b.position));

  function onscroll() {
    if (!strip) return;
    const width = strip.firstElementChild?.clientWidth ?? strip.clientWidth;
    current = Math.round(strip.scrollLeft / width);
  }

  async function add(status: string) {
    const title = draft.trim();
    addingTo = null;
    draft = '';
    if (title) await tasks.create({ path: folder, title, status });
  }

  function onAddKey(e: KeyboardEvent, status: string) {
    if (e.key === 'Enter') add(status);
    if (e.key === 'Escape') addingTo = null;
  }
</script>

<div class="strip" bind:this={strip} {onscroll}>
  {#each columns as col (col.value)}
    {@const cards = shown.filter((t) => t.status === col.value)}
    <section class="col" aria-label={col.label}>
      <h3><StatusDot value={col.value} size={9} />{col.label}<span class="n">{cards.length}</span></h3>
      {#each cards as t (t.id)}
        <a class="card" href={routes.task(t.id)}>
          <span class="tt">{t.title}</span>
          {#if t.dueDate}<span class="due" class:warn={isOverdue(t, tasks.day)}>{dueLabel(t.dueDate)}</span>{/if}
        </a>
      {/each}
      {#if addingTo === col.value}
        <!-- svelte-ignore a11y_autofocus -->
        <input class="add-in" placeholder="task title" autofocus bind:value={draft} onkeydown={(e) => onAddKey(e, col.value)} onblur={() => add(col.value)} />
      {:else}
        <button class="add" onclick={() => (addingTo = col.value)}><Icon name="plus" size={16} />add task</button>
      {/if}
    </section>
  {/each}
</div>
<div class="dots" aria-hidden="true">
  {#each columns as col, i (col.value)}<span class:on={i === current}></span>{/each}
</div>

<style>
  .strip {
    display: flex;
    gap: 16px;
    margin: 0 -20px;
    padding: 0 20px;
    overflow-x: auto;
    scroll-snap-type: x mandatory;
    scroll-padding: 0 20px;
    scrollbar-width: none;
  }

  .col {
    flex: 0 0 82%;
    display: flex;
    flex-direction: column;
    gap: 8px;
    scroll-snap-align: start;
  }

  h3 {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 36px;
    margin: 0;
    font: var(--text-ui-strong);
    font-size: 15px;
  }

  .n {
    font: var(--text-meta);
    color: var(--c-muted-2);
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 12px;
    border-radius: var(--radius-md);
    background: var(--c-chip);
    color: var(--text);
    text-decoration: none;
  }

  .card:active,
  .card:focus-visible {
    outline: none;
    background: var(--c-hover);
  }

  .tt {
    font: var(--text-ui);
    font-size: 15px;
    line-height: 22px;
  }

  .due {
    font: var(--text-caption);
    color: var(--c-muted-2);
  }

  .due.warn {
    color: var(--c-overdue);
  }

  .add {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 44px;
    padding: 0 6px;
    border: 0;
    background: transparent;
    font: var(--text-ui);
    font-size: 15px;
    color: var(--c-muted-2);
  }

  .add-in {
    height: 44px;
    padding: 0 12px;
    border: 1px solid var(--primary);
    border-radius: var(--radius-md);
    background: var(--c-page);
    font: var(--text-ui);
    font-size: 15px;
    color: var(--text);
    outline: none;
  }

  .dots {
    display: flex;
    justify-content: center;
    gap: 6px;
    margin: 16px 0;
  }

  .dots span {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--c-faint);
  }

  .dots span.on {
    background: var(--c-current);
  }
</style>
