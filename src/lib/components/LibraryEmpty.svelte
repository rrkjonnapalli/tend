<script lang="ts">
  // An empty Library area or search: one handwritten line and one hint, centred.
  let { title, hint }: { title: string; hint: string } = $props();
</script>

<div class="empty">
  <p class="title">{title}</p>
  <p class="hint">{hint}</p>
</div>

<style>
  .empty {
    flex: 1 1 auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    text-align: center;
  }

  .title {
    margin: 0;
    font: 600 36px/40px var(--font-hand);
    color: var(--c-muted-2);
  }

  .hint {
    margin: 0;
    font: var(--text-ui);
    color: var(--c-muted-2);
  }
</style>
