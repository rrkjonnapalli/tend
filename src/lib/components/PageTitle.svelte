<script lang="ts">
  // A page's title under its page bar, and the line under it: a folder or section name with its
  // count (lists), or a Settings section with what it holds. One style for every such title.
  let { title, meta = '' }: { title: string; meta?: string } = $props();
</script>

<div class="lt">
  <h1 class="heading">{title}</h1>
  {#if meta}<span class="meta">{meta}</span>{/if}
</div>

<style>
  .lt {
    flex: 0 0 auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 4px 0 8px;
  }

  .meta {
    font: var(--text-caption);
    color: var(--c-muted-2);
  }
</style>
