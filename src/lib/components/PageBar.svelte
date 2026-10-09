<script lang="ts">
  // The top row of every page: on the same line as the explorer's icon toolbar. Holds an
  // optional Back button, the breadcrumbs, and the page's actions on the right (in this order:
  // edit or read, move, delete, width). List pages put their title and count under it (ListTitle).
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  let {
    onback,
    children,
    actions,
  }: {
    /** Shows a Back button when set. */
    onback?: () => void;
    children: Snippet;
    actions?: Snippet;
  } = $props();
</script>

<div class="pbar caption" data-zone="bar">
  {#if onback}
    <button class="icon-btn sm back" aria-label="Back" title="Back" onclick={onback}><Icon name="back" size={13} /></button>
  {/if}
  {@render children()}
  <span class="grow"></span>
  {@render actions?.()}
</div>

<style>
  .pbar {
    position: sticky;
    top: 0;
    z-index: 3;
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    gap: 6px;
    height: 40px;
    margin: 0 -24px;
    padding: 0 24px;
    background: var(--c-page);
  }

  .back {
    margin-left: -6px;
  }

  .grow {
    flex: 1;
  }
</style>
