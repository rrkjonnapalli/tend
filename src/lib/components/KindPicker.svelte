<script lang="ts">
  // A word's kind as a small chip; it opens a search over the kinds (type to filter, ↑↓, Enter),
  // like the people picker, with a tick on the current one. Only a listed kind can be picked.
  import Icon from './Icon.svelte';
  import Popover from './Popover.svelte';
  import SearchPicker from './SearchPicker.svelte';
  import type { KindChoice, WordKind } from '#lib/types/word.type.ts';
  import { textMatches } from '#lib/util/search.ts';

  let { value, kinds, onpick }: { value: string; kinds: WordKind[]; onpick: (kind: string) => void } = $props();

  let open = $state(false);
  let chip = $state<HTMLButtonElement>();

  const label = $derived(kinds.find((k) => k.value === value)?.label ?? value.toLowerCase());
  const choices = $derived(kinds.map((k): KindChoice => ({ id: k.value, label: k.label })));

  const matches = (choice: KindChoice, query: string) => textMatches([choice.label], query);

  function pick(choice: KindChoice) {
    open = false;
    if (choice.id !== value) onpick(choice.id);
  }
</script>

<button bind:this={chip} class="chip" aria-haspopup="listbox" aria-expanded={open} aria-label="Kind: {label}" onclick={() => (open = !open)}>
  {label}<Icon name="chevronDown" size={12} />
</button>
{#if open}
  <Popover anchor={chip} onclose={() => (open = false)}>
    <SearchPicker items={choices} match={matches} onpick={pick} placeholder="Search kinds…" label="What is it" empty="No kind like that">
      {#snippet row(choice)}
        {choice.label}{#if choice.id === value}<span class="sub"><Icon name="check" size={13} /></span>{/if}
      {/snippet}
    </SearchPicker>
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

  .chip:hover {
    background: var(--c-hover);
  }
</style>
