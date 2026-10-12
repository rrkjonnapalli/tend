<script lang="ts">
  // The snap editor: a paper over the dimmed board, the same size for every paper so nothing jumps
  // when it is rotated. Top: × (Esc) cancels; pin to top, delete (editing only), ✓ (⌘↵) saves.
  // Bottom: rotate the paper, pick the stack, the sticky's colour, tags. Only the text is needed.
  import { untrack } from 'svelte';
  import Icon from './Icon.svelte';
  import Popover from './Popover.svelte';
  import SnapPaper from './SnapPaper.svelte';
  import SwatchPicker from './SwatchPicker.svelte';
  import TagEditor from './TagEditor.svelte';
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { focusOnMount } from '#lib/actions/focus.action.ts';
  import { papers, stickyColor } from '#lib/config/library.config.ts';
  import { PAPER } from '#lib/enums/snap.enum.ts';
  import { snaps } from '#lib/stores/snaps.svelte.ts';
  import type { Snap, SnapDraft } from '#lib/types/snap.type.ts';
  import { pushLayer } from '#lib/util/layers.ts';
  import { BOARD_NAME, NEW_STACK, stackName, stackTree } from '#lib/util/snaps.ts';

  let {
    snap,
    open,
    onsave,
    ondelete,
    onclose,
  }: {
    /** The snap to edit; none writes a new one. */
    snap: Snap | null;
    /** The stack the board shows: a new snap goes in it. */
    open: string | null;
    onsave: (draft: SnapDraft) => void;
    ondelete: () => void;
    onclose: () => void;
  } = $props();

  // The editor is remounted for each snap, so these start from the one it opened with.
  const start = untrack(() => snap);
  const where = untrack(() => open);
  let text = $state(start?.text ?? '');
  let paperAt = $state(Math.max(0, papers.findIndex((p) => p.paper === start?.paper)));
  let color = $state(start?.color ?? stickyColor);
  let stackId = $state<string | null>(start ? start.stackId : where);
  let pinned = $state(start?.pinned ?? false);
  let tags = $state<string[]>(start?.tags ?? []);
  let picking = $state<'stack' | 'color' | null>(null);
  let stackBtn = $state<HTMLButtonElement>();
  let colorBtn = $state<HTMLButtonElement>();
  let ink = $state<HTMLTextAreaElement>();

  const paper = $derived(papers[paperAt].paper);
  const next = $derived(papers[(paperAt + 1) % papers.length]);
  const isSticky = $derived(paper === PAPER.sticky);
  const tree = $derived(stackTree(snaps.stacks));
  const openStack = $derived(snaps.stacks.find((s) => s.id === where));
  const destName = $derived.by(() => {
    if (stackId === NEW_STACK) return 'New stack';
    const s = snaps.stacks.find((st) => st.id === stackId);
    return s ? stackName(s) : BOARD_NAME;
  });

  // Esc closes the editor once its pickers are closed (they are layers above it).
  $effect(() => pushLayer(() => onclose()));

  function rotate() {
    paperAt = (paperAt + 1) % papers.length;
    ink?.focus();
  }

  function pickStack(id: string | null) {
    stackId = id;
    picking = null;
    ink?.focus();
  }

  function pickColor(hex: string) {
    color = hex;
    picking = null;
    ink?.focus();
  }

  function save() {
    if (!text.trim()) {
      ink?.focus();
      return;
    }
    onsave({ text, paper, color, stackId, pinned, tags });
  }

  function onkeydown(e: KeyboardEvent) {
    const saves = e.key === 'Enter' && (e.metaKey || e.ctrlKey);
    if (!saves) return;
    e.preventDefault();
    save();
  }
</script>

<div class="veil">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="draft" class:torn={paper === PAPER.scrap} role="dialog" aria-label={start ? 'Edit snap' : 'New snap'} tabindex="-1" {onkeydown}>
    <SnapPaper {paper} {color} editor>
      <textarea use:plainText use:focusOnMount bind:this={ink} class="ink" aria-label="Snap text" placeholder="Write it down…" bind:value={text}></textarea>
      <div class="foot">
        <button class="pbtn" aria-label="Change paper to {next.name}" title="Change paper to {next.name}" onclick={rotate}><Icon name="rotate" size={17} /></button>
        <button bind:this={stackBtn} class="where" aria-label="Put in: {destName}" title="Put in: {destName}" aria-haspopup="listbox" aria-expanded={picking === 'stack'} onclick={() => (picking = picking === 'stack' ? null : 'stack')}>
          <Icon name="stack" size={15} /><span>{destName}</span>
        </button>
        {#if isSticky}
          <button bind:this={colorBtn} class="pbtn" aria-label="Sticky colour" title="Sticky colour" aria-expanded={picking === 'color'} onclick={() => (picking = picking === 'color' ? null : 'color')}>
            <span class="dot" style:background={color}></span>
          </button>
        {/if}
        <div class="tags"><TagEditor {tags} onchange={(t) => (tags = t)} /></div>
      </div>
    </SnapPaper>
    <span class="pin" aria-hidden="true"></span>
    <button class="pbtn corner left" aria-label="Cancel" title="Cancel (Esc)" onclick={onclose}><Icon name="close" size={17} /></button>
    <span class="corner right">
      <button class="pbtn" class:on={pinned} aria-label="Pin to top" title={pinned ? 'Pinned to top' : 'Pin to top'} aria-pressed={pinned} onclick={() => (pinned = !pinned)}><Icon name="pin" size={16} /></button>
      {#if start}
        <button class="pbtn del" aria-label="Delete snap" title="Delete snap" onclick={ondelete}><Icon name="trash" size={15} /></button>
      {/if}
      <button class="pbtn save" aria-label={start ? 'Save' : 'Add'} title="{start ? 'Save' : 'Add'} (⌘↵)" onclick={save}><Icon name="check" size={17} /></button>
    </span>
  </div>
</div>

{#if picking === 'stack'}
  <Popover anchor={stackBtn} placement="top-start" role="presentation" pad onclose={() => (picking = null)}>
    <div class="menu" role="listbox" aria-label="Put in stack">
      <button class="opt" class:on={stackId === null} role="option" aria-selected={stackId === null} onclick={() => pickStack(null)}>{BOARD_NAME} (loose)</button>
      {#each tree as { stack, depth } (stack.id)}
        <button class="opt" class:on={stackId === stack.id} role="option" aria-selected={stackId === stack.id} style:padding-left="{10 + depth * 14}px" onclick={() => pickStack(stack.id)}>{stackName(stack)}</button>
      {/each}
      <button class="opt new" class:on={stackId === NEW_STACK} role="option" aria-selected={stackId === NEW_STACK} onclick={() => pickStack(NEW_STACK)}>
        + New stack{openStack ? ` in ${stackName(openStack)}` : ''}
      </button>
    </div>
  </Popover>
{/if}
{#if picking === 'color' && colorBtn}
  <SwatchPicker anchor={colorBtn} {color} label="Sticky colour" onpick={pickColor} onclose={() => (picking = null)} />
{/if}

<style>
  /* Above the board, below popovers (30) so the stack, colour and tag pickers show over it. */
  .veil {
    position: absolute;
    inset: 0;
    z-index: 20;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--board-veil);
    backdrop-filter: blur(2px);
  }

  .draft {
    position: relative;
    display: flex;
    flex-direction: column;
    width: 340px;
    height: 300px;
    rotate: -1deg;
    outline: none;
  }

  .torn {
    filter: var(--paper-torn-shadow);
  }

  .ink {
    flex: 1 1 auto;
    width: 100%;
    min-height: 0;
    margin: 0;
    padding: 0;
    border: 0;
    outline: none;
    resize: none;
    background: transparent;
    font: inherit;
    color: var(--board-ink);
    overflow-y: auto;
  }

  .ink::placeholder {
    color: var(--board-ink-muted);
  }

  .foot {
    position: absolute;
    left: 12px;
    right: 16px;
    bottom: 24px;
    display: flex;
    align-items: center;
    gap: 2px;
    height: 30px;
    font: var(--text-meta);
  }

  .tags {
    min-width: 0;
    max-width: 170px;
    overflow: hidden;
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

  .pbtn {
    width: 30px;
    height: 30px;
    flex: none;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: transparent;
    color: var(--board-ink-muted);
    cursor: pointer;
  }

  .pbtn:hover,
  .pbtn:focus-visible {
    background: var(--board-hover);
    color: var(--board-ink);
  }

  .pbtn.on {
    color: var(--pin-on);
  }

  .pbtn.save {
    color: var(--primary);
  }

  .del:hover,
  .del:focus-visible {
    color: var(--danger);
  }

  .corner {
    position: absolute;
    top: 8px;
  }

  .left {
    left: 8px;
  }

  .right {
    right: 8px;
    display: flex;
  }

  .where {
    height: 30px;
    max-width: 120px;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 0 8px 0 6px;
    border: 0;
    border-radius: 15px;
    background: transparent;
    font: var(--text-meta);
    color: var(--board-ink-muted);
    cursor: pointer;
  }

  .where:hover,
  .where:focus-visible {
    background: var(--board-hover);
    color: var(--board-ink);
  }

  .where span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dot {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    box-shadow: inset 0 0 0 1px var(--border);
  }

  .menu {
    width: 250px;
    max-height: 220px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .opt {
    height: 30px;
    flex: none;
    padding: 0 10px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    text-align: left;
    font: var(--text-caption);
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: pointer;
  }

  .opt:hover,
  .opt:focus-visible {
    background: var(--c-hover);
  }

  .opt.on {
    background: var(--c-selected);
    color: var(--c-current-ink);
  }

  .opt.new {
    margin-top: 4px;
    border-top: 1px solid var(--border);
    border-radius: 0 0 var(--radius-control) var(--radius-control);
    color: var(--primary);
  }
</style>
