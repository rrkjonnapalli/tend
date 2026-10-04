<script lang="ts">
  // A task row's assignee: the person's initials, or a gray "U" when nobody is assigned. Click to
  // pick someone, or to unassign. It may sit inside a link row, so its clicks stay on the row.
  import Avatar from './Avatar.svelte';
  import PeoplePicker from './PeoplePicker.svelte';
  import { people } from '#lib/stores/people.svelte.ts';
  import { displayName } from '#lib/util/people.ts';
  import type { PersonRow } from '#lib/types/person.type.ts';

  let { value, onchange }: { value: string | null; onchange: (id: string | null) => void } = $props();

  const UNASSIGNED = 'Unassigned';

  let open = $state(false);
  let anchor = $state<HTMLElement>();
  const person = $derived(people.byId(value));
  const label = $derived(person ? `Assignee: ${displayName(person)}` : UNASSIGNED);

  function toggle(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    open = !open;
  }

  function choose(id: string | null) {
    open = false;
    if (id !== value) onchange(id);
  }

  function pick(p: PersonRow) {
    choose(p.id);
  }
</script>

<button bind:this={anchor} class="badge" data-no-drag aria-label={label} title={label} aria-haspopup="dialog" aria-expanded={open} onclick={toggle}>
  {#if person}
    <Avatar {person} />
  {:else}
    <span class="none">U</span>
  {/if}
</button>
{#if open}
  <PeoplePicker {anchor} onpick={pick} onclose={() => (open = false)}>
    {#snippet lead()}
      {#if person}<button class="opt unassign" onclick={() => choose(null)}>Unassign</button>{/if}
    {/snippet}
  </PeoplePicker>
{/if}

<style>
  .badge {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    margin-left: -2px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: transparent;
    cursor: pointer;
  }

  .badge:hover {
    background: var(--c-hover);
  }

  .none {
    display: inline-grid;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--c-chip);
    color: var(--c-muted-2);
    font: 700 9px/1 var(--font-ui);
  }

  .unassign {
    width: 100%;
    border-bottom: 1px solid var(--c-chip);
    color: var(--danger);
  }
</style>
