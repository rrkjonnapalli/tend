<script lang="ts">
  // A violet circle with a person's initials.
  import { initials, displayName } from '#lib/util/people.ts';
  import type { PersonName } from '#lib/types/person.type.ts';

  let { person, size = 20 }: { person: PersonName; size?: number } = $props();

  /** Initials take a little under half the circle's height. */
  const FONT_RATIO = 0.45;
  const fontSize = $derived(Math.round(size * FONT_RATIO));
</script>

<span class="av" style:width="{size}px" style:height="{size}px" style:font-size="{fontSize}px" title={displayName(person)}>
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
