<script lang="ts">
  // The bottom of a list while items are selected, in place of the menu button: Cut · (Rename or
  // Tag) · Delete (MD-Select, MD-Snap-Select).
  import Icon from '../Icon.svelte';
  import type { ClipAction } from '#lib/types/clip.type.ts';

  let { oncut, middle, ondelete }: { oncut: () => void; middle: ClipAction; ondelete: () => void } = $props();
</script>

<div class="bar" role="toolbar" aria-label="Selected items">
  <button onclick={oncut}><Icon name="cut" size={22} />Cut</button>
  <button disabled={middle.disabled} onclick={middle.onclick}><Icon name={middle.icon} size={22} />{middle.label}</button>
  <button class="danger" onclick={ondelete}><Icon name="trash" size={22} />Delete</button>
</div>

<style>
  .bar {
    position: fixed;
    left: 0;
    right: 0;
    bottom: var(--kb, 0px);
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
