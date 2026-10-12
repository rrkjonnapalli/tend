<script lang="ts">
  // One snap on the board: its paper with a push-pin (brass when pinned to top), the date and its
  // first tag. A click opens it; a press that moves drags it. Hover shows pin and delete.
  import Icon from './Icon.svelte';
  import SnapPaper from './SnapPaper.svelte';
  import { PAPER } from '#lib/enums/snap.enum.ts';
  import type { Snap } from '#lib/types/snap.type.ts';
  import { ago } from '#lib/util/dates.ts';
  import { dropKey, tagLabel } from '#lib/util/snaps.ts';

  let {
    snap,
    tilt,
    where,
    dragging,
    pairing,
    onopen,
    onpress,
    onpin,
    ondelete,
  }: {
    snap: Snap;
    tilt: number;
    /** "in Work rules", while searching across stacks. */
    where: string | null;
    dragging: boolean;
    /** Another snap is held over this one: dropping makes a stack of the two. */
    pairing: boolean;
    onopen: () => void;
    onpress: (e: PointerEvent) => void;
    onpin: () => void;
    ondelete: () => void;
  } = $props();

  const firstLine = $derived(snap.text.split('\n')[0]);
</script>

<div class="item" class:dragging class:pairing data-drop={dropKey('snap', snap.id)} style:rotate="{tilt}deg">
  {#if where}<span class="in">in {where}</span>{/if}
  <div class="note" class:top={snap.pinned} class:torn={snap.paper === PAPER.scrap}>
    <SnapPaper paper={snap.paper} color={snap.color}>
      <div class="txt">{snap.text}</div>
      <div class="foot"><span>{ago(snap.updatedAt)}</span><span class="tag">{tagLabel(snap.tags)}</span></div>
    </SnapPaper>
    <span class="pin" aria-hidden="true"></span>
    <button class="hit" aria-label="Edit snap: {firstLine}" onclick={onopen} onpointerdown={onpress}></button>
    <span class="tools">
      <button class="tool" class:on={snap.pinned} aria-label={snap.pinned ? 'Unpin from top' : 'Pin to top'} title={snap.pinned ? 'Unpin from top' : 'Pin to top'} aria-pressed={snap.pinned} onclick={onpin}>
        <Icon name="pin" size={13} />
      </button>
      <button class="tool del" aria-label="Delete snap: {firstLine}" title="Delete" onclick={ondelete}><Icon name="trash" size={13} /></button>
    </span>
  </div>
</div>

<style>
  .item {
    position: relative;
    display: flow-root;
    margin: 0 0 44px;
    touch-action: pan-y;
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

  .note {
    position: relative;
    display: flex;
    flex-direction: column;
    transition: translate 0.15s ease, rotate 0.18s ease;
  }

  .note:hover {
    translate: 0 -3px;
  }

  .torn {
    filter: var(--paper-torn-shadow);
  }

  /* A snap held over this one: this one shifts and a second sheet shows behind it. */
  .pairing .note {
    translate: 8px -4px;
    rotate: 2deg;
  }

  .pairing::before {
    content: '';
    position: absolute;
    inset: 0;
    background: var(--paper);
    box-shadow: var(--paper-shadow);
    rotate: -5deg;
    translate: -10px 6px;
  }

  .txt {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .foot {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: auto;
    font: var(--text-meta);
    color: var(--board-ink-muted);
  }

  .tag {
    margin-left: auto;
  }

  .pin {
    position: absolute;
    top: 8px;
    left: 50%;
    width: 14px;
    height: 14px;
    margin-left: -7px;
    border-radius: 50%;
    background: var(--pin-red);
    box-shadow: var(--pin-shadow);
  }

  .top .pin {
    width: 16px;
    height: 16px;
    margin-left: -8px;
    background: var(--pin-brass);
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
    z-index: 3;
    display: flex;
    opacity: 0;
    transition: opacity 0.12s ease;
  }

  .note:hover .tools,
  .tools:focus-within,
  .hit:focus-visible ~ .tools {
    opacity: 1;
  }

  /* Keyboard focus on the snap: it lifts, like on hover; never a ring. */
  .note:has(.hit:focus-visible) {
    translate: 0 -3px;
  }

  @media (hover: none) {
    .tools {
      opacity: 1;
    }
  }

  .tool {
    width: 22px;
    height: 22px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--board-ink-muted);
    cursor: pointer;
  }

  .tool:hover,
  .tool:focus-visible {
    background: var(--board-hover);
    color: var(--board-ink);
  }

  .tool.on {
    color: var(--pin-on);
  }

  .del:hover,
  .del:focus-visible {
    color: var(--danger);
  }
</style>
