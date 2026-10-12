<script lang="ts">
  // One stack on the board: a small pile under a binder clip, with its name (handwritten), the
  // first lines of its latest snap and a count. A click opens it; a press that moves drags it.
  // Hover shows rename and remove. Held over by something being dragged, its sheets fan out.
  import Icon from './Icon.svelte';
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { focusAndSelect } from '#lib/actions/focus.action.ts';
  import type { Stack } from '#lib/types/snap.type.ts';
  import { dropKey, stackName } from '#lib/util/snaps.ts';

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
    /** "in Work rules", while searching across stacks. */
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

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== 'Enter' && e.key !== 'Escape') return;
    e.preventDefault();
    e.stopPropagation();
    onrename(draft);
  }
</script>

<div class="item" class:dragging data-drop={dropKey('stack', stack.id)} style:rotate="{tilt}deg">
  {#if where}<span class="in">in {where}</span>{/if}
  <div class="stack" class:over>
    <span class="under u2"></span>
    <span class="under u1"></span>
    <span class="top">
      {#if renaming}<span class="name-space"></span>{:else}<span class="name">{name}</span>{/if}
      <span class="peek">{peek}</span>
      <span class="foot"><span>{summary}</span><span class="tag">stack</span></span>
    </span>
    <span class="clip" aria-hidden="true"></span>
    <button class="hit" aria-label="Open stack {name}, {summary}" onclick={onopen} onpointerdown={onpress}></button>
    {#if renaming}
      <input use:plainText use:focusAndSelect class="rename" aria-label="Stack name" placeholder="Name this stack" maxlength="40" bind:value={draft} {onkeydown} onblur={() => onrename(draft)} />
    {/if}
    <span class="tools">
      <button class="tool" aria-label="Rename stack {name}" title="Rename" onclick={() => { draft = stack.name; onstartrename(); }}><Icon name="edit" size={13} /></button>
      <button class="tool del" aria-label="Remove stack {name}, keeping its snaps" title="Remove stack (keeps its snaps)" onclick={onremove}><Icon name="trash" size={13} /></button>
    </span>
  </div>
</div>

<style>
  .item {
    position: relative;
    display: flow-root;
    margin: 0 0 30px;
    padding-top: 30px;
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

  .stack {
    position: relative;
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

  .u1 {
    background: var(--paper-kraft);
    transform: rotate(4deg) translate(6px, 5px);
    box-shadow: var(--paper-shadow);
  }

  .u2 {
    background: var(--paper);
    transform: rotate(-4deg) translate(-5px, 8px);
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
    box-shadow: var(--paper-shadow);
  }

  .stack:hover .top,
  .stack:has(.hit:focus-visible) .top {
    translate: 0 -3px;
  }

  .stack:hover .u1 {
    translate: 8px 0;
  }

  .stack:hover .u2 {
    translate: -8px 0;
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
    font: 600 30px/32px var(--font-hand);
  }

  .name-space {
    height: 32px;
  }

  .peek {
    max-height: 60px;
    overflow: hidden;
    white-space: pre-wrap;
    font: 400 13px/20px var(--font-mono);
    color: var(--board-ink-muted);
  }

  .foot {
    display: flex;
    gap: 8px;
    margin-top: auto;
    font: var(--text-meta);
    color: var(--board-ink-muted);
  }

  .tag {
    margin-left: auto;
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

  .hit {
    position: absolute;
    inset: 0;
    z-index: 2;
    padding: 0;
    border: 0;
    background: transparent;
    cursor: pointer;
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
    font: 600 28px/32px var(--font-hand);
    color: var(--board-ink);
  }

  .tools {
    position: absolute;
    top: 6px;
    right: 6px;
    z-index: 4;
    display: flex;
    opacity: 0;
    transition: opacity 0.12s ease;
  }

  .stack:hover .tools,
  .tools:focus-within,
  .hit:focus-visible ~ .tools {
    opacity: 1;
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

  .del:hover,
  .del:focus-visible {
    color: var(--danger);
  }
</style>
