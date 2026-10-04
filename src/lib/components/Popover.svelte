<script lang="ts">
  // A floating panel anchored to an element or a point. It flips to stay inside the window,
  // closes on Esc and on a click outside, and never grows the page.
  import type { Anchor, Placement } from '#lib/types/popover.type.ts';
  import type { Snippet } from 'svelte';
  import { pushLayer } from '#lib/util/layers.ts';
  import { positionFor } from '#lib/util/popover-position.ts';

  let {
    anchor,
    onclose,
    placement = 'bottom-start',
    offset = 4,
    label,
    role = 'dialog',
    pad = false,
    children,
  }: {
    anchor: Anchor | null | undefined;
    onclose: () => void;
    placement?: Placement;
    offset?: number;
    label?: string;
    role?: string;
    /** Inner padding of 4px top and bottom, for menus and option lists. */
    pad?: boolean;
    children: Snippet;
  } = $props();

  /** Before the first measure, so the popover never flashes at 0,0. */
  const OFFSCREEN = -9999;

  let el = $state<HTMLDivElement>();
  let pos = $state({ left: OFFSCREEN, top: OFFSCREEN });

  function rectOf(a: Anchor): { left: number; right: number; top: number; bottom: number } {
    if (a instanceof HTMLElement) return a.getBoundingClientRect();
    return { left: a.x, right: a.x, top: a.y, bottom: a.y };
  }

  function place() {
    if (!el || !anchor) return;
    pos = positionFor({
      rect: rectOf(anchor),
      size: { width: el.offsetWidth, height: el.offsetHeight },
      placement,
      offset,
      viewport: { width: window.innerWidth, height: window.innerHeight },
    });
  }

  $effect(() => {
    if (!el) return;
    place();
    const ro = new ResizeObserver(place);
    ro.observe(el);
    window.addEventListener('resize', place);
    return () => {
      ro.disconnect();
      window.removeEventListener('resize', place);
    };
  });

  $effect(() => pushLayer(() => onclose()));

  function onpointerdown(e: PointerEvent) {
    const target = e.target as Node;
    if (el?.contains(target)) return;
    if (anchor instanceof HTMLElement && anchor.contains(target)) return;
    onclose();
  }
</script>

<svelte:window onpointerdowncapture={onpointerdown} />

<div bind:this={el} class="popover" class:pad {role} aria-label={label} style:left="{pos.left}px" style:top="{pos.top}px">
  {@render children()}
</div>

<style>
  .popover {
    position: fixed;
    z-index: 30;
    max-height: calc(100vh - 16px);
    overflow: auto;
    background: var(--c-page);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text);
    font: 400 13px/18px var(--font-ui);
  }

  .pad {
    padding: 4px 0;
  }
</style>
