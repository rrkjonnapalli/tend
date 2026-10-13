<script lang="ts">
  // A word's Dictionary section: senses with examples, synonyms, antonyms and a link to the entry's
  // own page at its source (the credit Wiktionary's licence asks for), or a hint when there is nothing
  // (not looked up yet, not in the dictionary, a kind that isn't looked up). Shimmers while asking.
  import Chip from './Chip.svelte';
  import Section from './Section.svelte';
  import Icon from './Icon.svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import type { Dictionary } from '#lib/types/word.type.ts';

  let {
    dictionary,
    fetching,
    hint,
    meta,
    onrefetch,
  }: {
    dictionary: Dictionary | null;
    fetching: boolean;
    hint: string;
    /** "dictionaryapi.dev · fetched 3 Oct" */
    meta: string;
    /** Shown as a re-fetch icon when set. */
    onrefetch?: () => void;
  } = $props();

  const antonyms = $derived(dictionary?.antonyms ?? []);
  const sourceUrl = $derived(dictionary?.sourceUrl ?? null);
</script>

<Section title="Dictionary" extra={meta} indent={false}>
  {#snippet actions()}
    {#if sourceUrl}
      <button class="icon-btn sm" aria-label="Open this entry at its source" title="Open at the source" onclick={() => openUrl(sourceUrl)}><Icon name="open" size={13} /></button>
    {/if}
    {#if onrefetch}
      <button class="icon-btn sm" aria-label="Fetch again" title="Fetch again" onclick={onrefetch}><Icon name="reload" size={13} /></button>
    {/if}
  {/snippet}
  <div class="body">
    {#if fetching}
      <div class="skel" role="status" aria-label="Looking it up"><span style:width="72%"></span><span style:width="54%"></span><span style:width="38%"></span></div>
    {:else if dictionary}
      {#each dictionary.senses as sense, i (i)}
        <div class="sense">
          <span class="n">{i + 1}</span>
          <span class="def">{sense.definition}</span>
          {#if sense.example}<span class="ex">“{sense.example}”</span>{/if}
        </div>
      {/each}
      {#if dictionary.synonyms.length}
        <div class="syn">
          <span class="meta">synonyms</span>
          {#each dictionary.synonyms as s (s)}<Chip text={s} />{/each}
        </div>
      {/if}
      {#if antonyms.length}
        <div class="syn">
          <span class="meta">antonyms</span>
          {#each antonyms as a (a)}<Chip text={a} />{/each}
        </div>
      {/if}
    {:else}
      <p class="hint">{hint}</p>
    {/if}
  </div>
</Section>

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .sense {
    display: grid;
    grid-template-columns: 20px minmax(0, 1fr);
    gap: 2px 6px;
  }

  .n {
    font: var(--text-read-mono);
    color: var(--c-muted-2);
  }

  .def {
    font: var(--text-read);
  }

  .ex {
    grid-column: 2;
    font: var(--text-caption);
    color: var(--text-muted);
  }

  .syn {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
  }

  .meta {
    width: 64px;
    font: var(--text-meta);
    color: var(--c-muted-2);
  }

  .hint {
    margin: 0;
    font: var(--text-read);
    color: var(--text-muted);
  }

  .skel {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 4px 0;
  }

  .skel span {
    height: 12px;
    border-radius: 6px;
    background: linear-gradient(90deg, var(--c-chip), var(--c-hover), var(--c-chip));
    background-size: 200% 100%;
    animation: shimmer 1.1s linear infinite;
  }

  @keyframes shimmer {
    to {
      background-position: -200% 0;
    }
  }
</style>
