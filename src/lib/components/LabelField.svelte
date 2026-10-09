<script lang="ts">
  // A task's labels: colored chips with a red ✕, and "+" which opens the configured list to tick
  // labels on and off. Only labels from Settings → Tasks can be picked.
  import Chip from './Chip.svelte';
  import Icon from './Icon.svelte';
  import Popover from './Popover.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { nextIndex, prevIndex } from '#lib/util/list-nav.ts';

  let { taskId }: { taskId: string } = $props();

  let open = $state(false);
  let button = $state<HTMLButtonElement>();
  let hi = $state(0);

  const chosen = $derived(tasks.labelsOf(taskId));
  const chosenIds = $derived(new Set(chosen.map((l) => l.id)));
  const empty = $derived(tasks.labels.length === 0);

  function setLabels(ids: string[]) {
    tasks.setLabels(taskId, ids);
  }

  function toggle(id: string) {
    const ids = chosen.map((l) => l.id);
    if (chosenIds.has(id)) setLabels(ids.filter((x) => x !== id));
    else setLabels([...ids, id]);
  }

  function remove(id: string) {
    setLabels(chosen.map((l) => l.id).filter((x) => x !== id));
  }

  function show() {
    open = true;
    hi = 0;
  }

  /** ↑↓ move, Enter or Space ticks; the list stays open to pick several. */
  function onkeydown(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      hi = nextIndex(hi, tasks.labels.length);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      hi = prevIndex(hi);
    } else if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      const label = tasks.labels[hi];
      if (label) toggle(label.id);
    }
  }
</script>

<div class="labels" aria-label="Labels">
  {#each chosen as l (l.id)}
    <Chip text={l.name} color={l.color} removeLabel="Remove label {l.name}" onremove={() => remove(l.id)} />
  {/each}
  <button bind:this={button} class="add" aria-label="Add label" title="Add label" aria-haspopup="listbox" aria-expanded={open} onclick={show} {onkeydown}>+</button>
</div>
{#if open}
  <Popover anchor={button} role="listbox" label="Labels" pad onclose={() => (open = false)}>
    <div class="list">
      {#each tasks.labels as l, i (l.id)}
        <button class="opt" class:on={i === hi} role="option" aria-selected={chosenIds.has(l.id)} onclick={() => toggle(l.id)} onmouseenter={() => (hi = i)}>
          <span class="dot" style:background={l.color}></span>{l.name}
          {#if chosenIds.has(l.id)}<span class="ck"><Icon name="check" size={12} stroke={2.5} /></span>{/if}
        </button>
      {/each}
      {#if empty}<span class="none">No labels yet. Add them in Settings → Tasks.</span>{/if}
    </div>
  </Popover>
{/if}

<style>
  .labels {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    min-height: 26px;
  }

  .add {
    height: 22px;
    padding: 0 6px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--c-muted-2);
    font: var(--text-caption);
    cursor: pointer;
  }

  .add:hover {
    background: var(--c-hover);
    color: var(--text);
  }

  .list {
    min-width: 180px;
    display: flex;
    flex-direction: column;
  }

  .dot {
    flex: 0 0 8px;
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .ck {
    margin-left: auto;
    display: grid;
    color: var(--primary);
  }

  .none {
    max-width: 220px;
    padding: 6px 8px;
    font: var(--text-meta);
    color: var(--c-muted-2);
  }
</style>
