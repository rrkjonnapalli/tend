<script lang="ts">
  // The top row of every page: on the same line as the explorer's icon toolbar. Holds an
  // optional Back button, the breadcrumbs, a short count, and the page's actions on the right.
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  let {
    onback,
    meta = '',
    children,
    actions,
  }: {
    /** Shows a Back button when set. */
    onback?: () => void;
    /** Shown after the breadcrumbs, e.g. "12 tasks". */
    meta?: string;
    children: Snippet;
    actions?: Snippet;
  } = $props();
</script>

<div class="pbar caption">
  {#if onback}
    <button class="icon-btn sm back" aria-label="Back" title="Back" onclick={onback}><Icon name="back" size={13} /></button>
  {/if}
  {@render children()}
  {#if meta}<span class="meta">· {meta}</span>{/if}
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

  .meta {
    color: var(--c-muted-2);
    white-space: nowrap;
  }

  .grow {
    flex: 1;
  }
</style>
