<script lang="ts">
  // References on a phone (ML-Refs): title, then the link or the text, then tags and when it was
  // added. A row opens the reference; "Why I kept it" is there.
  import Chip from '../Chip.svelte';
  import Icon from '../Icon.svelte';
  import { REF_KIND } from '#lib/enums/ref.enum.ts';
  import type { Ref } from '#lib/types/ref.type.ts';
  import { ago } from '#lib/util/dates.ts';
  import { refTitle, shownUrl } from '#lib/util/refs.ts';

  let { refs, onopen }: { refs: Ref[]; onopen: (ref: Ref) => void } = $props();
</script>

<div class="phone-list">
  {#each refs as r (r.id)}
    <button class="wr" onclick={() => onopen(r)}>
      <span class="w">{refTitle(r)}</span>
      {#if r.kind === REF_KIND.link}
        <span class="m mono"><Icon name="open" size={13} />{shownUrl(r.value)}</span>
      {:else if r.title}
        <span class="m">{r.value}</span>
      {/if}
      <span class="foot">
        {#each r.tags as t (t)}<Chip text={t} />{/each}
        <span class="d">{ago(r.createdAt)}</span>
      </span>
    </button>
  {:else}
    <p class="empty">Nothing matches</p>
  {/each}
</div>

<style>
  .wr {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: calc(100% + 20px);
    margin: 0 -10px;
    padding: 10px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    text-align: left;
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
    display: flex;
    align-items: center;
    gap: 4px;
    max-width: 100%;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    font: var(--text-caption);
    color: var(--c-muted-2);
  }

  .m.mono {
    font: var(--text-mono-meta);
  }

  .foot {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 4px;
  }

  .d {
    margin-left: auto;
    font: var(--text-meta);
    color: var(--c-muted-2);
  }
</style>
