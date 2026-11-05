<script lang="ts">
  // Words on a phone (ML-Words): a collapsible Kinds section that filters the list, then each word
  // with its meaning on one line. A word opens on its own page.
  import Icon from '../Icon.svelte';
  import type { WordGroup } from '#lib/types/word.type.ts';
  import { routes } from '#lib/util/routes.ts';

  let { groups }: { groups: WordGroup[] } = $props();

  let open = $state(false);
  let kind = $state<string | null>(null);

  const picked = $derived(groups.find((g) => g.kind.value === kind) ?? null);
  const shown = $derived(
    (picked ? [picked] : groups).flatMap((g) => g.words).sort((a, b) => a.title.localeCompare(b.title, undefined, { sensitivity: 'base' })),
  );

  function pick(value: string | null) {
    kind = value;
    open = false;
  }
</script>

<div class="phone-list">
  <button class="fs" aria-expanded={open} onclick={() => (open = !open)}>
    <Icon name={open ? 'chevronDown' : 'chevronRight'} size={14} stroke={2.5} />Kinds<span class="cur">· {picked?.kind.label ?? 'all'}</span>
  </button>
  {#if open}
    <button class="kr" class:on={kind === null} onclick={() => pick(null)}>all</button>
    {#each groups as g (g.kind.value)}
      <button class="kr" class:on={kind === g.kind.value} onclick={() => pick(g.kind.value)}>{g.kind.label}<span class="n">{g.words.length}</span></button>
    {/each}
  {/if}
  <div class="gap"></div>
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
  .fs,
  .kr {
    display: flex;
    align-items: center;
    gap: 6px;
    width: calc(100% + 20px);
    height: 44px;
    margin: 0 -10px;
    padding: 0 10px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    text-align: left;
    color: var(--text);
  }

  .fs {
    font: var(--text-section);
  }

  .fs :global(svg),
  .cur,
  .n {
    color: var(--c-muted-2);
  }

  .cur {
    font: var(--text-ui);
  }

  .kr {
    padding-left: 34px;
    font: var(--text-ui);
    font-size: 15px;
  }

  .kr.on {
    background: var(--c-selected);
    color: var(--c-current-ink);
  }

  .n {
    margin-left: auto;
    font: var(--text-meta);
  }

  .gap {
    height: 12px;
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
  .wr:focus-visible,
  .fs:focus-visible,
  .kr:focus-visible {
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
