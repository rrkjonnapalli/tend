<script lang="ts">
  // One word, snap or reference as Home lists it: its title and one line under it. Opens it.
  import { goto } from '$app/navigation';
  import type { LibraryHit } from '#lib/types/library.type.ts';

  let { hit }: { hit: LibraryHit } = $props();
</script>

<button class="item" onclick={() => goto(hit.href)}>
  <b class:plain={hit.plain}>{hit.title}</b>
  {#if hit.sub}<span class:mono={hit.mono}>{hit.sub}</span>{/if}
</button>

<style>
  .item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    box-sizing: content-box;
    width: 100%;
    margin: 0 -10px;
    padding: 8px 10px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    text-align: left;
    cursor: pointer;
  }

  .item:hover,
  .item:focus-visible {
    background: var(--c-hover);
  }

  b,
  span {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  b {
    font: 500 15px/20px var(--font-heading);
    color: var(--text);
  }

  b.plain {
    font: var(--text-ui);
  }

  span {
    font: var(--text-caption);
    color: var(--text-muted);
  }

  span.mono {
    font: 400 12px/16px var(--font-mono);
  }
</style>
