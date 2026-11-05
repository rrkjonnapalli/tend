<script lang="ts">
  // Words on a phone (ML-Words): a kind dropdown (all kinds unless one is picked) that filters the
  // list, then each word with its meaning on one line. A word opens on its own page.
  import type { WordGroup } from '#lib/types/word.type.ts';
  import { routes } from '#lib/util/routes.ts';

  let { groups }: { groups: WordGroup[] } = $props();

  const ALL = '';

  let kind = $state(ALL);

  const picked = $derived(groups.find((g) => g.kind.value === kind) ?? null);
  const shown = $derived(
    (picked ? [picked] : groups).flatMap((g) => g.words).sort((a, b) => a.title.localeCompare(b.title, undefined, { sensitivity: 'base' })),
  );
</script>

<div class="phone-list">
  <select class="kind" aria-label="Kind" bind:value={kind}>
    <option value={ALL}>All kinds</option>
    {#each groups as g (g.kind.value)}<option value={g.kind.value}>{g.kind.label} ({g.words.length})</option>{/each}
  </select>
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
    height: 36px;
    margin: 4px 0 12px;
    padding: 0 10px;
    border: 0;
    border-radius: var(--radius-sm);
    background: var(--c-chip);
    font: var(--text-ui);
    font-size: 15px;
    color: var(--text);
  }

  .kind:focus-visible {
    outline: none;
    background: var(--c-hover);
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
