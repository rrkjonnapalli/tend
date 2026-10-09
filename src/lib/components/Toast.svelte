<script lang="ts">
  // The toast line. An error is announced at once and has a close button.
  import Icon from './Icon.svelte';
  import { toast } from '#lib/stores/toast.svelte.ts';
</script>

{#if toast.text}
  {#if toast.isError}
    <div class="toast err" role="alert">
      {toast.text}
      <button class="x" aria-label="Dismiss" title="Dismiss" onclick={toast.hide}><Icon name="close" size={11} stroke={2.25} /></button>
    </div>
  {:else}
    <div class="toast" role="status">{toast.text}</div>
  {/if}
{/if}

<style>
  .toast {
    position: fixed;
    left: 50%;
    bottom: 64px;
    z-index: 50;
    transform: translateX(-50%);
    padding: 6px 12px;
    border-radius: var(--radius-control);
    background: var(--text);
    color: var(--c-page);
    font: var(--text-caption);
    white-space: nowrap;
    pointer-events: none;
  }

  .toast.err {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    border: 1px solid var(--danger);
    background: var(--danger-soft);
    color: var(--on-danger-soft);
    pointer-events: auto;
  }

  .x {
    display: grid;
    padding: 0;
    border: 0;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
</style>
