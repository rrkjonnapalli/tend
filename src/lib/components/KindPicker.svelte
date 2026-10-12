<script lang="ts">
  // A word's kind as a small chip; it opens the list of kinds with a tick on the current one.
  import Icon from './Icon.svelte';
  import Popover from './Popover.svelte';
  import { focusOnMount } from '#lib/actions/focus.action.ts';
  import type { WordKind } from '#lib/types/word.type.ts';

  let { value, kinds, onpick }: { value: string; kinds: WordKind[]; onpick: (kind: string) => void } = $props();

  let open = $state(false);
  let chip = $state<HTMLButtonElement>();

  const label = $derived(kinds.find((k) => k.value === value)?.label ?? value.toLowerCase());

  function pick(kind: string) {
    open = false;
    chip?.focus();
    if (kind !== value) onpick(kind);
  }

  /** ↑↓ move between the options. */
  function onkeydown(e: KeyboardEvent) {
    const step = { ArrowDown: 1, ArrowUp: -1 }[e.key];
    if (!step) return;
    e.preventDefault();
    const options = [...(e.currentTarget as HTMLElement).querySelectorAll<HTMLButtonElement>('[role="option"]')];
    const at = options.indexOf(document.activeElement as HTMLButtonElement);
    const next = Math.min(Math.max(at + step, 0), options.length - 1);
    options[next]?.focus();
  }
</script>

<button bind:this={chip} class="chip" aria-haspopup="listbox" aria-expanded={open} aria-label="Kind: {label}" onclick={() => (open = !open)}>
  {label}<Icon name="chevronDown" size={12} />
</button>
{#if open}
  <Popover anchor={chip} role="presentation" pad onclose={() => (open = false)}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="opts" role="listbox" aria-label="What is it" tabindex="-1" {onkeydown}>
      {#each kinds as k (k.value)}
        {@const on = k.value === value}
        {#if on}
          <button class="opt on" role="option" aria-selected="true" use:focusOnMount onclick={() => pick(k.value)}>
            <span>{k.label}</span><Icon name="check" size={13} />
          </button>
        {:else}
          <button class="opt" role="option" aria-selected="false" onclick={() => pick(k.value)}><span>{k.label}</span></button>
        {/if}
      {/each}
    </div>
  </Popover>
{/if}

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    height: 22px;
    padding: 0 4px 0 8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: var(--c-chip);
    font: var(--text-caption);
    color: var(--text);
    cursor: pointer;
  }

  .chip:hover,
  .chip:focus-visible {
    background: var(--c-hover);
  }

  .opts {
    min-width: 170px;
    display: flex;
    flex-direction: column;
    outline: none;
  }

  .opt {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    height: 30px;
    padding: 0 8px 0 10px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    font: var(--text-caption);
    color: var(--text);
    text-align: left;
    cursor: pointer;
  }

  .opt:hover,
  .opt:focus-visible {
    background: var(--c-hover);
  }

  .opt.on {
    color: var(--c-current-ink);
  }
</style>
