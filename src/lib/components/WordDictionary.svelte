<script lang="ts">
  // A word's Dictionary section: senses with examples, synonyms, or a hint when there is nothing
  // (not looked up yet, not in the dictionary, a kind that isn't looked up). Shimmers while asking.
  import Section from './Section.svelte';
  import Icon from './Icon.svelte';
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
</script>

<Section title="Dictionary" extra={meta} indent={false}>
  {#snippet actions()}
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
          {#each dictionary.synonyms as s (s)}<span class="chip">{s}</span>{/each}
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
    font: 400 13px/22px var(--font-mono);
    color: var(--c-muted-2);
  }

  .def {
    font: 400 14px/22px var(--font-ui);
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

  .chip {
    height: 22px;
    padding: 0 8px;
    border-radius: var(--radius-sm);
    background: var(--c-chip);
    font: var(--text-caption);
    line-height: 22px;
  }

  .hint {
    margin: 0;
    font: 400 14px/22px var(--font-ui);
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
