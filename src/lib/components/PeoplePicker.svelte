<script lang="ts">
  // A popover with a people search: type to filter, ↑↓ to move, Enter or click to pick.
  // `footer` adds rows under the list (e.g. "Unassign").
  import type { Snippet } from 'svelte';
  import Avatar from './Avatar.svelte';
  import Popover from './Popover.svelte';
  import SearchPicker from './SearchPicker.svelte';
  import { people } from '#lib/stores/people.svelte.ts';
  import { displayName, personMatches } from '#lib/util/people.ts';
  import type { PersonRow } from '#lib/types/person.type.ts';

  let {
    anchor,
    onpick,
    onclose,
    footer,
  }: {
    anchor: HTMLElement | undefined;
    onpick: (person: PersonRow) => void;
    onclose: () => void;
    footer?: Snippet;
  } = $props();
</script>

<Popover {anchor} {onclose}>
  <SearchPicker items={people.people} match={personMatches} {onpick} placeholder="Search people…" label="People" empty="No people yet">
    {#snippet row(p)}
      <Avatar person={p} size={18} />{displayName(p)}<span class="sub">{p.email ?? p.phone ?? ''}</span>
    {/snippet}
  </SearchPicker>
  {@render footer?.()}
</Popover>
