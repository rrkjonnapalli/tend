<script lang="ts">
  // One stack on the board: a small pile under a binder clip, with its name (handwritten), the
  // first lines of its latest snap and a count. A click opens it; a press that moves drags it.
  // Hover shows rename and remove. Held over by something being dragged, its sheets fan out.
  import BoardFoot from './BoardFoot.svelte';
  import BoardItem from './BoardItem.svelte';
  import BoardTool from './BoardTool.svelte';
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { focusAndSelect } from '#lib/actions/focus.action.ts';
  import { BOARD_KIND } from '#lib/enums/snap.enum.ts';
  import type { Stack } from '#lib/types/snap.type.ts';
  import { stackName } from '#lib/util/snaps.ts';

  let {
    stack,
    tilt,
    where,
    summary,
    peek,
    dragging,
    over,
    renaming,
    onopen,
    onpress,
    onstartrename,
    onrename,
    onremove,
  }: {
    stack: Stack;
    tilt: number;
    /** "Work rules", while searching across stacks. */
    where: string | null;
    /** "4 snaps · 1 stack" */
    summary: string;
    peek: string;
    dragging: boolean;
    /** Something is held over it: dropping puts it in. */
    over: boolean;
    renaming: boolean;
    onopen: () => void;
    onpress: (e: PointerEvent) => void;
    onstartrename: () => void;
    /** The name typed (Enter, Esc or leaving the field all keep it). */
    onrename: (name: string) => void;
    onremove: () => void;
  } = $props();

  const name = $derived(stackName(stack));
  // svelte-ignore state_referenced_locally
  let draft = $state(stack.name);

  function startRename() {
    draft = stack.name;
    onstartrename();
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== 'Enter' && e.key !== 'Escape') return;
    e.preventDefault();
    e.stopPropagation();
    onrename(draft);
  }
</script>

<BoardItem kind={BOARD_KIND.stack} id={stack.id} {tilt} {where} {dragging} {over} label="Open stack {name}, {summary}" {onopen} {onpress}>
  <span class="pile" class:over>
    <span class="under u2"></span>
    <span class="under u1"></span>
    <span class="top">
      {#if renaming}<span class="name-space"></span>{:else}<span class="name">{name}</span>{/if}
      <span class="peek">{peek}</span>
      <BoardFoot note={summary} tag="stack" />
    </span>
    <span class="clip" aria-hidden="true"></span>
  </span>
  {#snippet field()}
    {#if renaming}
      <input use:plainText use:focusAndSelect class="rename" aria-label="Stack name" placeholder="Name this stack" maxlength="40" bind:value={draft} {onkeydown} onblur={() => onrename(draft)} />
    {/if}
  {/snippet}
  {#snippet tools()}
    <BoardTool icon="edit" label="Rename stack {name}" title="Rename" onclick={startRename} />
    <BoardTool icon="trash" label="Remove stack {name}, keeping its snaps" title="Remove stack (keeps its snaps)" danger onclick={onremove} />
  {/snippet}
</BoardItem>

<style>
  .pile {
    position: relative;
    display: block;
    height: 200px;
    color: var(--board-ink);
  }

  .under,
  .top,
  .clip {
    transition: translate 0.18s ease, rotate 0.18s ease;
  }

  .under {
    position: absolute;
    inset: 0;
  }

  /* On hover the card (BoardItem) lifts the top sheet and fans the two under it. */
  .u1 {
    background: var(--paper-kraft);
    transform: rotate(4deg) translate(6px, 5px);
    translate: var(--card-fan) 0;
    box-shadow: var(--paper-shadow);
  }

  .u2 {
    background: var(--paper);
    transform: rotate(-4deg) translate(-5px, 8px);
    translate: calc(-1 * var(--card-fan)) 0;
    box-shadow: var(--paper-shadow);
  }

  .top {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 30px 18px 14px;
    background: var(--paper);
    translate: 0 var(--card-lift);
    box-shadow: var(--paper-shadow);
  }

  /* Something held over it: the top sheet lifts, the sheets under it fan out. */
  .over .top,
  .over .clip {
    translate: 0 -10px;
  }

  .over .u1 {
    translate: 14px 4px;
    rotate: 3deg;
  }

  .over .u2 {
    translate: -14px 6px;
    rotate: -3deg;
  }

  .name {
    font: var(--text-hand-stack);
  }

  .name-space {
    height: 32px;
  }

  .peek {
    max-height: 60px;
    overflow: hidden;
    white-space: pre-wrap;
    font: var(--text-code);
    color: var(--board-ink-muted);
  }

  .clip {
    position: absolute;
    top: -6px;
    left: 50%;
    width: 48px;
    height: 20px;
    margin-left: -24px;
    border-radius: 3px 3px 2px 2px;
    background: var(--clip);
    box-shadow: var(--pin-shadow);
  }

  .clip::before,
  .clip::after {
    content: '';
    position: absolute;
    top: -12px;
    width: 12px;
    height: 14px;
    border: 2px solid var(--clip-wire);
    border-bottom: 0;
    border-radius: 7px 7px 0 0;
  }

  .clip::before {
    left: 7px;
  }

  .clip::after {
    right: 7px;
  }

  .rename {
    position: absolute;
    top: 28px;
    left: 14px;
    right: 14px;
    z-index: 5;
    height: 36px;
    padding: 0 4px;
    border: 0;
    border-bottom: 1px solid var(--board-ink);
    background: transparent;
    outline: none;
    font: var(--text-hand-stack-edit);
    color: var(--board-ink);
  }
</style>
