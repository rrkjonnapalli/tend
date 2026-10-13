<script lang="ts">
  // The small last line of a board item's paper: a note on the left, a tag on the right.
  let { note, tag }: { note: string; tag: string } = $props();
</script>

<span class="foot"><span>{note}</span><span class="tag">{tag}</span></span>

<style>
  .foot {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: auto;
    font: var(--text-meta);
    color: var(--board-ink-muted);
  }

  .tag {
    margin-left: auto;
  }
</style>
