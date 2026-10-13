<script lang="ts">
  // The path to the open stack, floating at the board's top left: "Snaps / Work rules / Code
  // reviews". Each step but the last opens that level, and takes drops to move things up to it.
  import { DROP_KIND } from '#lib/enums/snap.enum.ts';
  import type { Stack } from '#lib/types/snap.type.ts';
  import { BOARD_NAME, dropKey, stackName } from '#lib/util/snaps.ts';

  let {
    path,
    over,
    onopen,
  }: {
    /** From the board's first stack down to the open one. */
    path: Stack[];
    /** The `data-drop` key held over, to mark that step. */
    over: string | null;
    onopen: (id: string | null) => void;
  } = $props();

  const steps = $derived([{ id: null, name: BOARD_NAME }, ...path.slice(0, -1).map((s) => ({ id: s.id, name: stackName(s) }))]);
  const here = $derived(path.length ? stackName(path[path.length - 1]) : '');
  const keyOf = (id: string | null) => dropKey(DROP_KIND.crumb, id);
</script>

<nav class="trail" aria-label="Stack path">
  {#each steps as step (step.id)}
    <button class="crumb" class:on={over === keyOf(step.id)} data-drop={keyOf(step.id)} onclick={() => onopen(step.id)}>{step.name}</button>
    <span class="sep" aria-hidden="true">/</span>
  {/each}
  <b class="here">{here}</b>
</nav>

<style>
  .trail {
    position: sticky;
    top: 12px;
    z-index: 6;
    display: inline-flex;
    align-items: center;
    gap: 2px;
    margin: 12px 0 0 16px;
    padding: 2px 10px 2px 6px;
    border: 1px solid var(--board-edge);
    border-radius: var(--radius-md);
    background: var(--paper);
  }

  .crumb {
    height: 24px;
    padding: 0 6px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    font: var(--text-caption);
    color: var(--board-ink-muted);
    cursor: pointer;
  }

  .crumb:hover,
  .crumb:focus-visible {
    background: var(--board-drop);
    color: var(--board-ink);
  }

  .crumb.on {
    background: var(--board-drop);
    color: var(--board-ink);
  }

  .sep {
    font: var(--text-caption);
    color: var(--c-faint);
  }

  .here {
    padding-left: 4px;
    font: var(--text-trail-here);
    color: var(--board-ink);
  }
</style>
