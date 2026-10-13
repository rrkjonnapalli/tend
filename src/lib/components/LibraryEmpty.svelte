<script lang="ts">
  // An empty Library area or search: one handwritten line and one hint, centred. On the Snaps
  // board (which stays light in the dark theme) it takes the board's own colours.
  let { title, hint, board = false }: { title: string; hint: string; board?: boolean } = $props();
</script>

<div class="empty" class:board>
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
    font: var(--text-hand-lg);
    color: var(--c-muted-2);
  }

  .hint {
    margin: 0;
    font: var(--text-ui);
    color: var(--c-muted-2);
  }

  .board .title {
    color: var(--board-title);
  }

  .board .hint {
    color: var(--board-hint);
  }
</style>
