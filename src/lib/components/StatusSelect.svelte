<script lang="ts">
  // Status dropdown: the configured statuses, current one ticked.
  import Icon from './Icon.svelte';
  import Popover from './Popover.svelte';
  import StatusDot from './StatusDot.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';

  let { value, onchange }: { value: string; onchange: (value: string) => void } = $props();

  let open = $state(false);
  let button = $state<HTMLButtonElement>();
  let hi = $state(0);
  const current = $derived(tasks.status(value));

  function pick(v: string) {
    open = false;
    if (v !== value) onchange(v);
  }

  /** Moves the highlight by `by`, stopping at the first and last status. */
  function moveHighlight(by: number) {
    const last = tasks.statuses.length - 1;
    const next = hi + by;
    hi = Math.max(0, Math.min(last, next));
  }

  function onkeydown(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      moveHighlight(e.key === 'ArrowDown' ? 1 : -1);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      pick(tasks.statuses[hi].value);
    }
  }

  /** Opens or closes the list; it opens on the current status (or the first if unknown). */
  function toggle() {
    open = !open;
    const currentIndex = tasks.statuses.findIndex((s) => s.value === value);
    hi = Math.max(0, currentIndex);
  }

  /** Opens the list (the "Set status…" command). */
  export function show() {
    if (!open) toggle();
    button?.focus();
  }
</script>

<button bind:this={button} class="ghost" aria-haspopup="listbox" aria-expanded={open} onclick={toggle} {onkeydown}>
  <StatusDot {value} />{current?.label ?? value}<span class="chev"><Icon name="chevronDown" size={10} stroke={2.5} /></span>
</button>
{#if open}
  <Popover anchor={button} role="listbox" label="Status" pad onclose={() => (open = false)}>
    {#each tasks.statuses as s, i (s.value)}
      <button class="opt st" class:on={i === hi} role="option" aria-selected={s.value === value} onclick={() => pick(s.value)}>
        <StatusDot value={s.value} />{s.label}
        {#if s.value === value}<span class="ck"><Icon name="check" size={12} stroke={2.5} /></span>{/if}
      </button>
    {/each}
  </Popover>
{/if}

<style>
  .ghost {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 28px;
    padding: 0 8px;
    margin-left: -8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    font: 400 14px/20px var(--font-ui);
    cursor: pointer;
  }

  .ghost:hover {
    background: var(--c-hover);
  }

  .chev {
    display: grid;
    color: var(--c-muted-2);
  }

  .st {
    min-width: 148px;
  }

  .ck {
    margin-left: auto;
    display: grid;
    color: var(--primary);
  }
</style>
