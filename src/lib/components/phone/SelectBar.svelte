<script lang="ts">
  // The top of a list while rows are selected (MD-Select): Cancel · "N selected" · Select all.
  import { clip } from '#lib/stores/clip.svelte.ts';

  let { all }: { all: string[] } = $props();

  const count = $derived(clip.selecting?.ids.length ?? 0);
</script>

<div class="bar">
  <button class="tb" onclick={clip.stop}>Cancel</button>
  <span class="n">{count} selected</span>
  <button class="tb act" onclick={() => clip.selectAll(all)}>Select all</button>
</div>

<style>
  .bar {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    height: 56px;
  }

  .n {
    font: var(--text-section);
    font-size: 17px;
  }

  .tb {
    justify-self: start;
    height: 44px;
    padding: 0;
    border: 0;
    background: transparent;
    font: var(--text-ui);
    font-size: 15px;
    color: var(--text-muted);
  }

  .tb.act {
    justify-self: end;
    color: var(--primary);
  }

  .tb:focus-visible {
    outline: none;
    text-decoration: underline;
  }
</style>
