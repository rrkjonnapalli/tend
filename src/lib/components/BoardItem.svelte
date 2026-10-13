<script lang="ts">
  // The card every board item shares: its tilt and drop key, "in <stack>" while searching, a
  // button over the whole card (a click opens it, a press that moves drags it) and the tools that
  // show in its corner on hover. SnapItem and StackItem put their paper or pile inside.
  import type { Snippet } from 'svelte';
  import { BOARD_KIND, type BoardKind } from '#lib/enums/snap.enum.ts';
  import { dropKey } from '#lib/util/snaps.ts';
  import { lightSurface } from '#lib/util/theme.ts';

  let {
    kind,
    id,
    tilt,
    where,
    dragging,
    over,
    label,
    onopen,
    onpress,
    children,
    field,
    tools,
    light = false,
  }: {
    kind: BoardKind;
    id: string;
    tilt: number;
    /** "Work rules", while searching across stacks. */
    where: string | null;
    dragging: boolean;
    /** Something is held over it: the hover look is off, the item shows its own drop look. */
    over: boolean;
    /** The card button's accessible name. */
    label: string;
    onopen: () => void;
    onpress: (e: PointerEvent) => void;
    children: Snippet;
    /** A field over the card (a stack's name while it is typed). */
    field?: Snippet;
    /** BoardTool buttons. */
    tools: Snippet;
    /** A sticky's card: it stays light in the dark theme, and so do the tools on it. */
    light?: boolean;
  } = $props();
</script>

<div
  class="item"
  class:snap={kind === BOARD_KIND.snap}
  class:stack={kind === BOARD_KIND.stack}
  class:dragging
  class:over
  data-drop={dropKey(kind, id)}
  style:rotate="{tilt}deg">
  {#if where}<span class="in">in {where}</span>{/if}
  <div class="card" data-surface={lightSurface(light)}>
    {@render children()}
    <button class="hit" aria-label={label} onclick={onopen} onpointerdown={onpress}></button>
    {@render field?.()}
    <span class="tools">{@render tools()}</span>
  </div>
</div>

<style>
  .item {
    position: relative;
    display: flow-root;
    margin: 0 0 44px;
    touch-action: pan-y;
  }

  /* Room above a stack for its clip and the sheets fanned under it. */
  .item.stack {
    margin-bottom: 30px;
    padding-top: 30px;
  }

  .item.dragging {
    opacity: 0.3;
  }

  /* The lifted copy that follows the pointer (util/drag-ghost.ts). */
  .item:global(.ghost) {
    filter: var(--ghost-shadow);
  }

  .in {
    display: block;
    margin: 0 0 8px 2px;
    font: var(--text-meta);
    color: var(--board-hint);
  }

  /* --card-lift: how far the card rises on hover or keyboard focus (never a ring).
     --card-fan: how far a pile's lower sheets spread on hover. What is inside reads them. */
  .card {
    --card-lift: 0px;
    --card-fan: 0px;
    position: relative;
  }

  .card:hover {
    --card-lift: -3px;
    --card-fan: 8px;
  }

  .card:has(.hit:focus-visible) {
    --card-lift: -3px;
  }

  .over .card {
    --card-lift: 0px;
    --card-fan: 0px;
  }

  /* A snap rises whole; a stack moves its own sheets (StackItem). */
  .snap .card {
    translate: 0 var(--card-lift);
    transition: translate 0.15s ease;
  }

  .hit {
    position: absolute;
    inset: 0;
    z-index: 2;
    padding: 0;
    border: 0;
    background: transparent;
    cursor: pointer;
  }

  .tools {
    position: absolute;
    top: 4px;
    right: 4px;
    z-index: 4;
    display: flex;
    opacity: 0;
    transition: opacity 0.12s ease;
  }

  .stack .tools {
    top: 6px;
    right: 6px;
  }

  .card:hover .tools,
  .tools:focus-within,
  .hit:focus-visible ~ .tools {
    opacity: 1;
  }

  @media (hover: none) {
    .tools {
      opacity: 1;
    }
  }
</style>
