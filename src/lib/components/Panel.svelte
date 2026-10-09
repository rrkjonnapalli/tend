<script lang="ts">
  // The quick-entry panel shared by Move to…, quick create and the ⌘K palette:
  // a small header line, rows, and a hint footer.
  import type { Snippet } from 'svelte';
  import Modal from './Modal.svelte';

  let {
    label,
    onclose,
    head,
    foot,
    children,
  }: { label: string; onclose: () => void; head: Snippet; foot?: Snippet; children: Snippet } = $props();
</script>

<Modal {label} {onclose}>
  <div class="head">{@render head()}</div>
  {@render children()}
  {#if foot}<div class="foot">{@render foot()}</div>{/if}
</Modal>

<style>
  .head {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 30px;
    padding: 0 14px;
    border-bottom: 1px solid var(--c-chip);
    font: var(--text-meta);
    color: var(--c-muted-2);
  }

  .foot {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 30px;
    padding: 0 14px;
    border-top: 1px solid var(--c-chip);
    font: var(--text-meta);
    color: var(--c-muted-2);
  }

  .head :global(.k),
  .foot :global(.r) {
    margin-left: auto;
  }

  .head :global(.k) {
    font: 400 11px/16px var(--font-mono);
    color: var(--c-muted-2);
  }

  .foot :global(.note) {
    color: var(--c-muted-2);
  }

  .foot :global(.err) {
    color: var(--danger);
  }
</style>
