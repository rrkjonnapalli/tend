<script lang="ts">
  // One reference in the list: title · reference · why · tags · added. The row opens it for
  // editing; a link's open icon (over the Reference column) opens the page instead.
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Icon from './Icon.svelte';
  import { REF_KIND } from '#lib/enums/ref.enum.ts';
  import type { Ref } from '#lib/types/ref.type.ts';
  import { ago } from '#lib/util/dates.ts';
  import { openableUrl, refTitle, shownUrl } from '#lib/util/refs.ts';

  let { ref, onopen }: { ref: Ref; onopen: () => void } = $props();

  const title = $derived(refTitle(ref));
  const isLink = $derived(ref.kind === REF_KIND.link);
</script>

<div class="row">
  <button class="hit" data-row aria-label="Edit reference: {title}" onclick={onopen}>
    <span class="t">{title}</span>
    <span class="ref" class:url={isLink}>{isLink ? shownUrl(ref.value) : ref.value}</span>
    <span class="why">{ref.why}</span>
    <span class="tags">
      {#each ref.tags as tag (tag)}<span class="tag">{tag}</span>{/each}
    </span>
    <span class="added">{ago(ref.createdAt)}</span>
  </button>
  {#if isLink}
    <button class="icon-btn sm open" aria-label="Open {title}" title="Open link" onclick={() => openUrl(openableUrl(ref.value))}>
      <Icon name="open" size={13} />
    </button>
  {/if}
</div>

<style>
  .row {
    display: grid;
    grid-template-columns: var(--grid-ref);
    column-gap: 14px;
    margin: 0 -8px;
  }

  .hit {
    grid-column: 1 / -1;
    grid-row: 1;
    display: grid;
    grid-template-columns: subgrid;
    align-items: center;
    min-height: 42px;
    padding: 6px 8px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    text-align: left;
    color: var(--text);
    cursor: pointer;
  }

  .hit:hover,
  .hit:focus-visible {
    background: var(--c-hover);
  }

  .open {
    grid-column: 2;
    grid-row: 1;
    align-self: center;
    justify-self: start;
    margin-left: 4px;
  }

  .t,
  .ref,
  .why {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .t {
    font: var(--text-ui);
  }

  .ref {
    font: var(--text-caption);
  }

  /* Room for the open icon that sits over this column. */
  .ref.url {
    padding-left: 24px;
    font: var(--text-meta);
    font-family: var(--font-mono);
    color: var(--text-muted);
  }

  .why {
    font: var(--text-caption);
    color: var(--text-muted);
  }

  .tags {
    display: flex;
    gap: 4px;
    overflow: hidden;
  }

  .tag {
    flex: none;
    height: 22px;
    padding: 0 8px;
    border-radius: var(--radius-sm);
    background: var(--c-nav-active-soft);
    font: var(--text-caption);
    line-height: 22px;
    color: var(--c-current-ink);
  }

  .added {
    font: var(--text-meta);
    color: var(--c-muted-2);
    text-align: right;
  }
</style>
