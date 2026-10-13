<script lang="ts">
  // The Words list on the left: grouped by kind (in kind order, only kinds with words),
  // alphabetical within a group. While adding, a "new …" row at the top mirrors the title.
  // ↑↓ move through the words and the entry on the right follows the cursor.
  import type { WordGroup } from '#lib/types/word.type.ts';

  let {
    groups,
    selected,
    adding,
    onpick,
    onfollow,
  }: {
    groups: WordGroup[];
    selected: string | null;
    /** The new entry's row text, or null when not adding. */
    adding: string | null;
    onpick: (id: string) => void;
    /** The keyboard cursor landed on this word: show it, without a new step in history. */
    onfollow: (id: string) => void;
  } = $props();

  /** Only a row reached with the keys (a click shows no focus ring state and is picked by its
   *  click). Not while adding: moving the cursor mustn't drop the new word being typed. */
  function onfocus(e: FocusEvent, id: string) {
    const byKeyboard = (e.currentTarget as HTMLElement).matches(':focus-visible');
    if (byKeyboard && adding === null && id !== selected) onfollow(id);
  }
</script>

<aside class="list" aria-label="Words" data-pane="side">
  <div class="rows" data-zone="rows">
    {#if adding !== null}<div class="row on new">{adding}</div>{/if}
    {#each groups as g (g.kind.value)}
      <div class="group">{g.kind.label}<span class="count">{g.words.length}</span></div>
      {#each g.words as w (w.id)}
        <button class="row" class:on={adding === null && w.id === selected} data-row onfocus={(e) => onfocus(e, w.id)} onclick={() => onpick(w.id)}>{w.title}</button>
      {/each}
    {/each}
  </div>
</aside>

<style>
  .list {
    width: 232px;
    flex: none;
    display: flex;
    flex-direction: column;
    min-height: 0;
    border-right: 1px solid var(--border);
  }

  .rows {
    flex: 1 1 auto;
    overflow: auto;
    padding: 8px 0 12px;
  }

  .group {
    display: flex;
    align-items: center;
    height: 28px;
    margin-top: 8px;
    padding: 0 12px 0 14px;
    font: var(--text-ui);
    color: var(--text);
  }

  .group:first-child {
    margin-top: 0;
  }

  .count {
    margin-left: auto;
    font: var(--text-meta);
    color: var(--c-muted-2);
  }

  .row {
    display: block;
    width: 100%;
    height: 28px;
    padding: 0 12px 0 32px;
    overflow: hidden;
    border: 0;
    background: transparent;
    text-align: left;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: var(--text-ui);
    line-height: 28px;
    color: var(--text);
    cursor: pointer;
  }

  .row:hover,
  .row:focus-visible {
    background: var(--c-hover);
  }

  .row.on {
    background: var(--c-selected);
    color: var(--c-current-ink);
  }

  .new {
    font-style: italic;
    cursor: default;
  }
</style>
