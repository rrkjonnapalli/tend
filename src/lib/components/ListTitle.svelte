<script lang="ts">
  // A list page's title under its page bar: the folder (or section) name and a short count.
  let { title, meta }: { title: string; meta: string } = $props();
</script>

<div class="lt">
  <h1 class="heading">{title}</h1>
  <span class="meta">{meta}</span>
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
