<script lang="ts">
  // The assignee: a person chip with a red ✕, or "+ assign" which opens a people search.
  import Icon from './Icon.svelte';
  import Avatar from './Avatar.svelte';
  import PeoplePicker from './PeoplePicker.svelte';
  import { displayName } from '#lib/util/people.ts';
  import type { PersonRow } from '#lib/types/person.type.ts';
  import { people } from '#lib/stores/people.svelte.ts';
  import { routes } from '#lib/util/routes.ts';

  let { value, onchange }: { value: string | null; onchange: (id: string | null) => void } = $props();

  let open = $state(false);
  let anchor = $state<HTMLElement>();
  const person = $derived(people.byId(value));

  function pick(p: PersonRow) {
    open = false;
    onchange(p.id);
  }
</script>

<span class="wrap" bind:this={anchor}>
  {#if person}
    <span class="who">
      <Avatar {person} />
      <a href={routes.person(person.id)}>{displayName(person)}</a>
      <button class="x" aria-label="Unassign" title="Unassign" onclick={() => onchange(null)}><Icon name="close" size={9} stroke={3} /></button>
    </span>
  {:else}
    <button class="assign" onclick={() => (open = true)}>+ assign</button>
  {/if}
</span>
{#if open}
  <PeoplePicker {anchor} onpick={pick} onclose={() => (open = false)} />
{/if}

<style>
  .wrap {
    display: inline-flex;
  }

  .who {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 26px;
    padding: 0 4px;
    border-radius: 13px;
    background: var(--c-nav-active-soft);
    font: var(--text-caption);
  }

  .who a {
    text-decoration: none;
  }

  .x {
    width: 18px;
    height: 18px;
    padding: 0;
    border: 0;
    border-radius: 9px;
    background: transparent;
    color: var(--danger);
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .x:hover {
    background: var(--danger-soft);
  }

  .assign {
    height: 28px;
    padding: 0 8px;
    margin-left: -8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--c-muted-2);
    font: var(--text-ui);
    cursor: pointer;
  }

  .assign:hover {
    background: var(--c-hover);
  }
</style>
