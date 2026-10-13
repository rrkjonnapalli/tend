<script lang="ts">
  // One choice from a fixed list, shown as a small chip (a word's kind, the dictionary in
  // Settings). It opens the choices with a tick on the current one: under a search field (type to
  // filter, ↑↓, Enter), like the people picker, or as a plain list when there are only a few
  // (`search` off; ↑↓ and Enter still work). Only a listed choice can be picked.
  import Icon from './Icon.svelte';
  import Popover from './Popover.svelte';
  import SearchPicker from './SearchPicker.svelte';
  import type { Choice } from '#lib/types/ui.type.ts';
  import { textMatches } from '#lib/util/search.ts';

  let {
    value,
    choices,
    what,
    search = true,
    field = false,
    max,
    narrow = false,
    onpick,
  }: {
    value: string;
    choices: Choice[];
    /** What is being chosen, for the accessible names: "Kind", "Dictionary". */
    what: string;
    /** Off for a list too short to need filtering. */
    search?: boolean;
    /** A field-sized control, for a setting on its own line; else the small chip. */
    field?: boolean;
    /** How many matches the search lists; a long list (years) scrolls. */
    max?: number;
    /** A narrow list, for choices a few characters long (years). */
    narrow?: boolean;
    onpick: (id: string) => void;
  } = $props();

  let open = $state(false);
  let chip = $state<HTMLButtonElement>();

  const label = $derived(choices.find((c) => c.id === value)?.label ?? value.toLowerCase());

  const matches = (choice: Choice, query: string) => textMatches([choice.label], query);

  function pick(choice: Choice) {
    open = false;
    if (choice.id !== value) onpick(choice.id);
  }
</script>

<button bind:this={chip} class="chip" class:field aria-haspopup="listbox" aria-expanded={open} aria-label="{what}: {label}" onclick={() => (open = !open)}>
  <span class="now">{label}</span><Icon name="chevronDown" size={field ? 14 : 12} />
</button>
{#snippet ticked(choice: Choice)}
  {choice.label}{#if choice.id === value}<span class="sub"><Icon name="check" size={13} /></span>{/if}
{/snippet}

{#if open && search}
  <Popover anchor={chip} onclose={() => (open = false)}>
    <div class="search" class:narrow>
      <SearchPicker items={choices} match={matches} onpick={pick} {max} placeholder="Search…" label={what} empty={narrow ? 'None' : 'Nothing like that'}>
        {#snippet row(choice)}{@render ticked(choice)}{/snippet}
      </SearchPicker>
    </div>
  </Popover>
{:else if open}
  <Popover anchor={chip} role="listbox" label={what} pad onclose={() => (open = false)}>
    <div class="list" class:field>
      {#each choices as choice (choice.id)}
        <button class="opt" role="option" aria-selected={choice.id === value} onclick={() => pick(choice)}>{@render ticked(choice)}</button>
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

  .chip:hover {
    background: var(--c-hover);
  }

  .search.narrow {
    --picker-w: var(--choice-narrow-w);
  }

  /* As a field: the size and border of a text field, the chevron at its right end. */
  .chip.field {
    justify-content: space-between;
    gap: 12px;
    width: var(--choice-field-w);
    height: 32px;
    padding: 0 8px 0 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    background: var(--c-page);
    font: var(--text-ui);
  }

  .chip.field:hover {
    background: var(--c-hover);
  }

  .now {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* The field's list is as wide as the field, with roomier rows. */
  .list.field {
    min-width: var(--choice-field-w);
  }

  .list.field .opt {
    min-height: 32px;
    font: var(--text-ui);
  }

  /* The tick sits at the row's right end, as in the search list. */
  .opt .sub {
    margin-left: auto;
    padding-left: 12px;
    color: var(--c-muted-2);
  }
</style>
