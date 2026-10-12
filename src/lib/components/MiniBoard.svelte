<script lang="ts">
  // Home's small pinboard: the latest snaps, each pinned at a fixed spot. A snap opens the board
  // where it is.
  import { goto } from '$app/navigation';
  import LibraryEmpty from './LibraryEmpty.svelte';
  import { miniSpots, stickyColor } from '#lib/config/library.config.ts';
  import { PAPER } from '#lib/enums/snap.enum.ts';
  import type { Snap } from '#lib/types/snap.type.ts';
  import { routes } from '#lib/util/routes.ts';
  import { stickyTint } from '#lib/util/snaps.ts';

  let {
    snaps,
    loaded,
  }: {
    /** At most as many as there are spots. */
    snaps: Snap[];
    /** False until the board is read, so it isn't called empty too soon. */
    loaded: boolean;
  } = $props();

  const pinned = $derived(
    snaps.slice(0, miniSpots.length).map((snap, i) => {
      const sticky = snap.paper === PAPER.sticky;
      return { snap, spot: miniSpots[i], sticky, tint: sticky ? stickyTint(snap.color ?? stickyColor) : null };
    }),
  );
</script>

<div class="mini">
  {#each pinned as { snap, spot, sticky, tint } (snap.id)}
    <button
      class="paper"
      class:sticky
      class:slip={snap.paper === PAPER.slip}
      style:left="{spot.left}px"
      style:top="{spot.top}px"
      style:rotate="{spot.tilt}deg"
      style:--band={tint?.band}
      style:--tint={tint?.sheet}
      onclick={() => goto(routes.libraryStack(snap.stackId))}>
      <span class="pin" aria-hidden="true"></span>
      <span class="text">{snap.text}</span>
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
    padding: 22px 14px 10px;
    border: 0;
    background: var(--paper);
    box-shadow: var(--paper-shadow);
    color: var(--board-ink);
    text-align: left;
    cursor: pointer;
  }

  .slip {
    background: var(--paper-kraft);
  }

  .sticky {
    background: linear-gradient(var(--band) 0 12px, var(--tint) 12px);
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
    font: 400 12px/20px var(--font-mono);
  }

  .sticky .text {
    font: 500 19px/23px var(--font-hand);
  }

  .pin {
    position: absolute;
    top: 7px;
    left: 50%;
    width: 11px;
    height: 11px;
    margin-left: -5px;
    border-radius: 50%;
    background: var(--pin-red);
    box-shadow: var(--pin-shadow);
  }
</style>
