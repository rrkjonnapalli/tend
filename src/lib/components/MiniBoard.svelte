<script lang="ts">
  // Home's small pinboard: the latest snaps, each pinned at a fixed spot on its own paper. A snap
  // opens the board where it is.
  import { goto } from '$app/navigation';
  import LibraryEmpty from './LibraryEmpty.svelte';
  import Pin from './Pin.svelte';
  import SnapPaper from './SnapPaper.svelte';
  import { miniSpots } from '#lib/config/library.config.ts';
  import { PAPER, PAPER_SIZE } from '#lib/enums/snap.enum.ts';
  import type { Snap } from '#lib/types/snap.type.ts';
  import { routes } from '#lib/util/routes.ts';

  let {
    snaps,
    loaded,
  }: {
    /** At most as many as there are spots. */
    snaps: Snap[];
    /** False until the board is read, so it isn't called empty too soon. */
    loaded: boolean;
  } = $props();

  const pinned = $derived(snaps.slice(0, miniSpots.length).map((snap, i) => ({ snap, spot: miniSpots[i] })));
</script>

<div class="mini">
  {#each pinned as { snap, spot } (snap.id)}
    <button
      class="paper"
      class:torn={snap.paper === PAPER.scrap}
      style:left="{spot.left}px"
      style:top="{spot.top}px"
      style:rotate="{spot.tilt}deg"
      onclick={() => goto(routes.libraryStack(snap.stackId))}>
      <SnapPaper paper={snap.paper} color={snap.color} size={PAPER_SIZE.mini}>
        <span class="text">{snap.text}</span>
      </SnapPaper>
      <Pin mini />
    </button>
  {/each}
  {#if loaded && pinned.length === 0}
    <LibraryEmpty title="Nothing pinned yet" hint="Press + and write a snap." board />
  {/if}
</div>

<style>
  .mini {
    position: relative;
    display: flex;
    height: 300px;
    border: 1px solid var(--board-edge);
    border-radius: var(--radius-md);
    background-color: var(--board-bg);
    background-image: radial-gradient(var(--board-dot) 1px, transparent 1.3px);
    background-size: 22px 22px;
    overflow: hidden;
  }

  .paper {
    position: absolute;
    width: 150px;
    padding: 0;
    border: 0;
    background: transparent;
    text-align: left;
    cursor: pointer;
    transition: translate 0.15s ease;
  }

  /* Hover and keyboard focus: the paper lifts, as on the board (the focus fill is hidden behind
     it); never a ring. */
  .paper:hover,
  .paper:focus-visible {
    translate: 0 -3px;
  }

  .torn {
    filter: var(--paper-torn-shadow);
  }

  /* A long snap shows its first lines only. */
  .text {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    overflow: hidden;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
</style>
