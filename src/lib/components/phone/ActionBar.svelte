<script lang="ts">
  // The bottom of a list while rows are selected (MD-Select): Cut · Rename · Delete, in place of
  // the menu button. Rename works on one item at a time.
  import Icon from '../Icon.svelte';
  import { clip } from '#lib/stores/clip.svelte.ts';

  let { oncut, onrename, ondelete }: { oncut: () => void; onrename: () => void; ondelete: () => void } = $props();

  const one = $derived(clip.selecting?.ids.length === 1);
</script>

<div class="bar" role="toolbar" aria-label="Selected items">
  <button onclick={oncut}><Icon name="cut" size={22} />Cut</button>
  <button disabled={!one} onclick={onrename}><Icon name="edit" size={22} />Rename</button>
  <button class="danger" onclick={ondelete}><Icon name="trash" size={22} />Delete</button>
</div>

<style>
  .bar {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 30;
    display: flex;
    justify-content: space-around;
    padding: 8px 0 calc(8px + env(safe-area-inset-bottom));
    border-top: 1px solid var(--border);
    background: var(--c-page);
  }

  button {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    min-width: 72px;
    padding: 4px 8px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    font: var(--text-caption);
    color: var(--text);
  }

  button:disabled {
    color: var(--c-faint);
  }

  .danger {
    color: var(--danger);
  }

  button:focus-visible,
  button:active {
    outline: none;
    background: var(--c-hover);
  }
</style>
