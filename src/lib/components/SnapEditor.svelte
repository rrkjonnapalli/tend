<script lang="ts">
  // The snap editor: a paper over the dimmed board, the same size for every paper so nothing jumps
  // when it is rotated. Top: × (Esc) cancels; pin to top, delete (editing only), ✓ (⌘↵) saves.
  // Bottom: rotate the paper, pick the stack, the sticky's colour, tags. Only the text is needed.
  import { untrack } from 'svelte';
  import Icon from './Icon.svelte';
  import Pin from './Pin.svelte';
  import Popover from './Popover.svelte';
  import SearchPicker from './SearchPicker.svelte';
  import SnapPaper from './SnapPaper.svelte';
  import SwatchPicker from './SwatchPicker.svelte';
  import TagEditor from './TagEditor.svelte';
  import { pageActions } from '#lib/stores/page-actions.svelte.ts';
  import { label as keyLabel } from '#lib/util/keys.ts';
  import { lightSurface } from '#lib/util/theme.ts';
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { focusOnMount } from '#lib/actions/focus.action.ts';
  import { papers, stickyColor } from '#lib/config/library.config.ts';
  import { PAPER, PAPER_SIZE, SNAP_PICKER, type SnapPicker } from '#lib/enums/snap.enum.ts';
  import { snaps } from '#lib/stores/snaps.svelte.ts';
  import type { Snap, SnapDraft, StackChoice } from '#lib/types/snap.type.ts';
  import { pushLayer } from '#lib/util/layers.ts';
  import { NEW_STACK, stackChoiceMatches, stackChoices, whereLabel } from '#lib/util/snaps.ts';

  let {
    snap,
    filled = '',
    open,
    onsave,
    ondelete,
    onclose,
  }: {
    /** The snap to edit; none writes a new one. */
    snap: Snap | null;
    /** A new snap's text, handed over from Home. */
    filled?: string;
    /** The stack the board shows: a new snap goes in it. */
    open: string | null;
    onsave: (draft: SnapDraft) => void;
    ondelete: () => void;
    onclose: () => void;
  } = $props();

  // The editor is remounted for each snap, so these start from the one it opened with.
  const start = untrack(() => snap);
  const where = untrack(() => open);
  let text = $state(start?.text ?? untrack(() => filled));
  let paperAt = $state(Math.max(0, papers.findIndex((p) => p.paper === start?.paper)));
  let color = $state(start?.color ?? stickyColor);
  let stackId = $state<string | null>(start ? start.stackId : where);
  let pinned = $state(start?.pinned ?? false);
  let tags = $state<string[]>(start?.tags ?? []);
  let picking = $state<SnapPicker | null>(null);
  let stackBtn = $state<HTMLButtonElement>();
  let colorBtn = $state<HTMLButtonElement>();
  let ink = $state<HTMLTextAreaElement>();

  const paper = $derived(papers[paperAt].paper);
  const nextAt = $derived((paperAt + 1) % papers.length);
  const next = $derived(papers[nextAt]);
  const isSticky = $derived(paper === PAPER.sticky);
  const isLined = $derived(paper === PAPER.lined);
  const choices = $derived(stackChoices(snaps.stacks, where));
  const destName = $derived.by(() => {
    if (stackId === NEW_STACK) return 'New stack';
    return whereLabel(stackId, snaps.stacks);
  });

  // Esc closes the editor once its pickers are closed (they are layers above it).
  $effect(() => pushLayer(() => onclose()));

  function rotate() {
    paperAt = nextAt;
    ink?.focus();
  }

  // ⌘S again, while this is open, turns to the next paper (see commands.ts).
  $effect(() => pageActions.provide({ 'snap.paper': rotate }));

  function toggle(picker: SnapPicker) {
    picking = picking === picker ? null : picker;
  }

  /** Closing a picker hands the keyboard back to the text, so Esc and ⌘↵ keep working. */
  function closePicker() {
    picking = null;
    ink?.focus();
  }

  function pickStack(choice: StackChoice) {
    stackId = choice.stackId;
    closePicker();
  }

  function pickColor(hex: string) {
    color = hex;
    closePicker();
  }

  function setTags(next: string[]) {
    tags = next;
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
  <div class="draft" class:torn={paper === PAPER.scrap} class:lined={isLined} data-surface={lightSurface(isSticky)} role="dialog" aria-label={start ? 'Edit snap' : 'New snap'} tabindex="-1" {onkeydown}>
    <SnapPaper {paper} {color} size={PAPER_SIZE.editor}>
      <textarea use:plainText use:focusOnMount bind:this={ink} class="ink" class:ruled={isLined} aria-label="Snap text" placeholder="Write it down…" bind:value={text}></textarea>
      <div class="foot">
        <button class="pbtn" aria-label="Change paper to {next.name}" title="Change paper to {next.name} ({keyLabel('mod+s')})" onclick={rotate}><Icon name="rotate" size={17} /></button>
        <button bind:this={stackBtn} class="where" aria-label="Put in: {destName}" title="Put in: {destName}" aria-haspopup="listbox" aria-expanded={picking === SNAP_PICKER.stack} onclick={() => toggle(SNAP_PICKER.stack)}>
          <Icon name="stack" size={15} /><span>{destName}</span>
        </button>
        {#if isSticky}
          <button bind:this={colorBtn} class="pbtn" aria-label="Sticky colour" title="Sticky colour" aria-expanded={picking === SNAP_PICKER.color} onclick={() => toggle(SNAP_PICKER.color)}>
            <span class="dot" style:background={color}></span>
          </button>
        {/if}
        <div class="tags"><TagEditor {tags} onchange={setTags} /></div>
      </div>
    </SnapPaper>
    <Pin />
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

{#if picking === SNAP_PICKER.stack}
  <Popover anchor={stackBtn} placement="top-start" label="Put in stack" onclose={closePicker}>
    <SearchPicker items={choices} match={stackChoiceMatches} onpick={pickStack} placeholder="Search stacks…" label="Put in stack" empty="No stack by that name">
      {#snippet row(choice)}
        <span class="choice" class:new={choice.id === NEW_STACK}>{choice.label}</span>
        {#if choice.stackId === stackId}<span class="sub"><Icon name="check" size={14} /></span>{/if}
      {/snippet}
    </SearchPicker>
  </Popover>
{/if}
{#if picking === SNAP_PICKER.color && colorBtn}
  <SwatchPicker anchor={colorBtn} {color} label="Sticky colour" onpick={pickColor} onclose={closePicker} />
{/if}

<style>
  /* Above the board, below popovers (45) so the stack, colour and tag pickers show over it. */
  /* Over the whole window: the editor opens on any page. */
  .veil {
    position: fixed;
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

  /* Lined paper scrolls with its text: the field spans the sheet and draws the rules itself, so
     every line of text stays sitting on one. At the top they meet the sheet's own rules. */
  .ruled {
    flex: none;
    width: calc(100% + 62px);
    height: 240px;
    margin: 0 -18px 0 -44px;
    padding: 0 18px 0 44px;
    background-color: var(--paper);
    background-image:
      linear-gradient(90deg, transparent 33px, var(--paper-margin) 33px, var(--paper-margin) 34px, transparent 34px),
      repeating-linear-gradient(transparent 0 23px, var(--paper-rule) 23px 24px);
    background-attachment: local;
    /* No scrollbar: its gutter would cut the rules short of the paper's edge. */
    scrollbar-width: none;
  }

  .ruled::-webkit-scrollbar {
    display: none;
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

  /* On lined paper the buttons sit on a line of their own, like the text: the first (with the
     pin) and the last, with the text on the ten between. */
  .lined .corner {
    top: 0;
  }

  .lined .foot {
    bottom: 6px;
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

  /* A long stack path is cut short, not wrapped. */
  .choice {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .new {
    color: var(--primary);
  }
</style>
