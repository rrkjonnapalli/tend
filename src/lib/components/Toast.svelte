<script lang="ts">
  import { toast } from '#lib/stores/toast.svelte.ts';
</script>

{#if toast.text}
  <div class="toast" role="status">{toast.text}</div>
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
</style>
