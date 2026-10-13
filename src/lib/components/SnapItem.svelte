<script lang="ts">
  // One snap on the board: its paper with a push-pin (brass when pinned to top), the date and its
  // first tag. A click opens it; a press that moves drags it. Hover shows pin and delete.
  import BoardFoot from './BoardFoot.svelte';
  import BoardItem from './BoardItem.svelte';
  import BoardTool from './BoardTool.svelte';
  import Pin from './Pin.svelte';
  import SnapPaper from './SnapPaper.svelte';
  import { BOARD_KIND, PAPER } from '#lib/enums/snap.enum.ts';
  import type { Snap } from '#lib/types/snap.type.ts';
  import { ago } from '#lib/util/dates.ts';
  import { tagLabel } from '#lib/util/snaps.ts';

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
    /** "Work rules", while searching across stacks. */
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
  const pinLabel = $derived(snap.pinned ? 'Unpin from top' : 'Pin to top');
</script>

<BoardItem kind={BOARD_KIND.snap} id={snap.id} {tilt} {where} {dragging} over={pairing} label="Edit snap: {firstLine}" {onopen} {onpress}>
  <div class="spot" class:pairing>
    <div class="note" class:torn={snap.paper === PAPER.scrap}>
      <SnapPaper paper={snap.paper} color={snap.color}>
        <div class="txt">{snap.text}</div>
        <BoardFoot note={ago(snap.updatedAt)} tag={tagLabel(snap.tags)} />
      </SnapPaper>
      <Pin top={snap.pinned} />
    </div>
  </div>
  {#snippet tools()}
    <BoardTool icon="pin" label={pinLabel} title={pinLabel} pressed={snap.pinned} onclick={onpin} />
    <BoardTool icon="trash" label="Delete snap: {firstLine}" title="Delete" danger onclick={ondelete} />
  {/snippet}
</BoardItem>

<style>
  .spot,
  .note {
    position: relative;
  }

  .note {
    transition: translate 0.15s ease, rotate 0.18s ease;
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
</style>
