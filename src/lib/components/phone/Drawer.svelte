<script lang="ts">
  // The phone's left drawer: the folder tree (Notes, Tasks) or the Settings sections slide in over
  // the page. It opens from the page's tree button (ui.drawer) or a swipe from the left edge, and
  // closes on the veil, Esc, Back, or as soon as anything in it navigates.
  import type { Snippet } from 'svelte';
  import { afterNavigate } from '$app/navigation';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { pushLayer } from '#lib/util/layers.ts';

  let { label, children }: { label: string; children: Snippet } = $props();

  /** A swipe that starts this close to the left edge and travels this far opens the drawer. */
  const EDGE_PX = 24;
  const SWIPE_PX = 60;

  let startX: number | null = null;

  afterNavigate(() => (ui.drawer = false));

  $effect(() => {
    if (!ui.drawer) return;
    return pushLayer(() => (ui.drawer = false));
  });

  function ontouchstart(e: TouchEvent) {
    const x = e.touches[0].clientX;
    startX = !ui.drawer && x <= EDGE_PX ? x : null;
  }

  function ontouchmove(e: TouchEvent) {
    if (startX === null) return;
    if (e.touches[0].clientX - startX > SWIPE_PX) {
      ui.drawer = true;
      startX = null;
    }
  }
</script>

<svelte:window {ontouchstart} {ontouchmove} />

<button class="veil" class:open={ui.drawer} tabindex="-1" aria-hidden="true" onclick={() => (ui.drawer = false)}></button>
<div class="drawer" class:open={ui.drawer} role="dialog" aria-modal="true" aria-label={label} inert={!ui.drawer}>
  {@render children()}
</div>

<style>
  .veil {
    position: fixed;
    inset: 0;
    z-index: 50;
    padding: 0;
    border: 0;
    background: var(--c-scrim);
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.25s;
  }

  .veil.open {
    opacity: 1;
    pointer-events: auto;
  }

  .drawer {
    position: fixed;
    top: 0;
    bottom: 0;
    left: 0;
    z-index: 51;
    display: flex;
    flex-direction: column;
    width: min(320px, 86vw);
    padding: env(safe-area-inset-top) 0 env(safe-area-inset-bottom);
    border-right: 1px solid var(--border);
    background: var(--c-page);
    transform: translateX(-100%);
    transition: transform 0.28s cubic-bezier(0.2, 0.8, 0.2, 1);
  }

  .drawer.open {
    transform: none;
  }

  /* The desktop explorer fills the drawer instead of keeping its pane width. */
  .drawer :global(.explorer),
  .drawer :global(.side) {
    flex: 1 1 auto;
    border-right: 0;
  }

  /* Rows big enough to tap: the explorer's and the Settings sections'. */
  .drawer :global(.sec),
  .drawer :global(.tree [data-row]) {
    flex-basis: 44px;
    min-height: 44px;
    font-size: 15px;
  }

  @media (prefers-reduced-motion: reduce) {
    .veil,
    .drawer {
      transition: none;
    }
  }
</style>
