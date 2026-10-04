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

  function onkeydown(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      hi = Math.max(0, Math.min(tasks.statuses.length - 1, hi + (e.key === 'ArrowDown' ? 1 : -1)));
    } else if (e.key === 'Enter') {
      e.preventDefault();
      pick(tasks.statuses[hi].value);
    }
  }

  function toggle() {
    open = !open;
    hi = Math.max(0, tasks.statuses.findIndex((s) => s.value === value));
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
