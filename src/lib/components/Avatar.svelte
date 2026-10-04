<script lang="ts">
  // A violet circle with a person's initials.
  import { initials, displayName } from '#lib/util/people.ts';
  import type { PersonRow } from '#lib/types/person.type.ts';

  let { person, size = 20 }: { person: Pick<PersonRow, 'firstName' | 'lastName' | 'nickname'>; size?: number } = $props();
</script>

<span class="av" style:width="{size}px" style:height="{size}px" style:font-size="{Math.round(size * 0.45)}px" title={displayName(person)}>
  {initials(person)}
</span>

<style>
  .av {
    flex: 0 0 auto;
    display: inline-grid;
    place-items: center;
    border-radius: 50%;
    background: var(--accent);
    color: var(--c-page);
    font-family: var(--font-ui);
    font-weight: 700;
    line-height: 1;
    letter-spacing: 0.02em;
  }
</style>
