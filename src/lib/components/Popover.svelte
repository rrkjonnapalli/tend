<script lang="ts">
  // A floating panel anchored to an element or a point. It flips to stay inside the window,
  // closes on Esc and on a click outside, and never grows the page. It is moved to <body>, so a
  // transformed or clipping parent (a dialog, a rotated paper) can't shift or cut it. Its choices
  // (options, menu items) take ↑↓, and focus left inside returns to the anchor when it closes.
  import type { Anchor, Placement } from '#lib/types/popover.type.ts';
  import type { Snippet } from 'svelte';
  import { optionNav } from '#lib/actions/option-nav.action.ts';
  import { portal } from '#lib/actions/portal.action.ts';
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

  const opener = () => (anchor instanceof HTMLElement ? anchor : null);

  function onpointerdown(e: PointerEvent) {
    const target = e.target as Node;
    if (el?.contains(target)) return;
    if (anchor instanceof HTMLElement && anchor.contains(target)) return;
    onclose();
  }
</script>

<svelte:window onpointerdowncapture={onpointerdown} />

<div bind:this={el} use:portal={opener} use:optionNav class="popover" class:pad {role} aria-label={label} style:left="{pos.left}px" style:top="{pos.top}px">
  {@render children()}
</div>

<style>
  .popover {
    position: fixed;
    /* Above a dialog (41), below a toast (50). */
    z-index: 45;
    max-height: calc(100vh - 16px);
    overflow: auto;
    background: var(--c-page);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text);
    font: var(--text-caption);
  }

  .pad {
    padding: 4px 0;
  }
</style>
