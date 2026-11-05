<script lang="ts">
  // Words on a phone (ML-Words): a kind dropdown (all kinds unless one is picked) that filters the
  // list, then each word with its meaning on one line. A word opens on its own page.
  import type { WordGroup } from '#lib/types/word.type.ts';
  import ChoicePicker from '../ChoicePicker.svelte';
  import { routes } from '#lib/util/routes.ts';

  let { groups }: { groups: WordGroup[] } = $props();

  const ALL = '';

  let kind = $state(ALL);

  const choices = $derived([{ id: ALL, label: 'All kinds' }, ...groups.map((g) => ({ id: g.kind.value, label: `${g.kind.label} (${g.words.length})` }))]);
  const picked = $derived(groups.find((g) => g.kind.value === kind) ?? null);
  const shown = $derived(
    (picked ? [picked] : groups).flatMap((g) => g.words).sort((a, b) => a.title.localeCompare(b.title, undefined, { sensitivity: 'base' })),
  );
</script>

<div class="phone-list">
  <div class="kind">
    <ChoicePicker value={kind} {choices} what="Kind" search={false} onpick={(id) => (kind = id)} />
  </div>
  {#each shown as w (w.id)}
    <a class="wr" href={routes.libraryWord(w.id)}>
      <span class="w">{w.title}</span>
      {#if w.meaning}<span class="m">{w.meaning}</span>{/if}
    </a>
  {:else}
    <p class="empty">No words match</p>
  {/each}
</div>

<style>
  .kind {
    margin: 4px 0 12px;
  }

  .wr {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 1px;
    width: calc(100% + 20px);
    min-height: 54px;
    margin: 0 -10px;
    padding: 6px 10px;
    border-radius: var(--radius-control);
    text-decoration: none;
  }

  .wr:active,
  .wr:focus-visible {
    outline: none;
    background: var(--c-hover);
  }

  .w {
    font: var(--text-ui);
    font-size: 15px;
    line-height: 22px;
    color: var(--text);
  }

  .m {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: var(--text-caption);
    color: var(--c-muted-2);
  }
</style>
