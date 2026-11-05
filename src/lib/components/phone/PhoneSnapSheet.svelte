<script lang="ts">
  // A snap on a phone, new or opened (MD-Snap-New): the paper, a sticky's colour, the text on that
  // paper, the stack and the tags; an opened one can also be pinned to the top, cut or deleted.
  // The phone's form of SnapEditor: same props, and SnapDialogs saves it the same way.
  import { untrack } from 'svelte';
  import Sheet from './Sheet.svelte';
  import SnapPaper from '../SnapPaper.svelte';
  import ChoicePicker from '../ChoicePicker.svelte';
  import TagEditor from '../TagEditor.svelte';
  import Icon from '../Icon.svelte';
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { papers, phoneStickyColors, stickyColor } from '#lib/config/library.config.ts';
  import { PAPER, PAPER_SIZE } from '#lib/enums/snap.enum.ts';
  import { clip } from '#lib/stores/clip.svelte.ts';
  import { snaps } from '#lib/stores/snaps.svelte.ts';
  import type { Paper } from '#lib/enums/snap.enum.ts';
  import type { Snap, SnapDraft } from '#lib/types/snap.type.ts';
  import { stackChoices } from '#lib/util/snaps.ts';

  let {
    snap,
    filled = '',
    open,
    onsave,
    ondelete,
    onclose,
  }: {
    snap: Snap | null;
    filled?: string;
    open: string | null;
    onsave: (draft: SnapDraft) => void;
    ondelete: () => void;
    onclose: () => void;
  } = $props();

  // Remounted for each snap, so these start from the one it opened with.
  const start = untrack(() => snap);
  const where = untrack(() => open);
  let text = $state(start?.text ?? untrack(() => filled));
  let paper = $state<Paper>(start?.paper ?? papers[0].paper);
  let color = $state(start?.color ?? stickyColor);
  let stackId = $state<string | null>(start ? start.stackId : where);
  let pinned = $state(start?.pinned ?? false);
  let tags = $state<string[]>(start?.tags ?? []);

  const isSticky = $derived(paper === PAPER.sticky);
  const choices = $derived(stackChoices(snaps.stacks, where));
  const picked = $derived(choices.find((c) => c.stackId === stackId)?.id ?? choices[0].id);
  /** A colour picked on desktop that isn't among the phone's five stays offered first. */
  const colors = $derived(phoneStickyColors.includes(color) ? phoneStickyColors : [color, ...phoneStickyColors]);

  function pickStack(id: string) {
    stackId = choices.find((c) => c.id === id)?.stackId ?? null;
  }

  function save() {
    onsave({ text, paper, color, stackId, pinned, tags });
  }

  function cut() {
    if (!start) return;
    clip.cutOne('snaps', start.id, start.stackId ?? '');
    onclose();
  }
</script>

<Sheet title={start ? 'Snap' : 'New snap'} action={start ? 'Save' : 'Pin'} disabled={!text.trim()} onaction={save} {onclose}>
  <div class="papers" role="radiogroup" aria-label="Paper">
    {#each papers as p (p.paper)}
      <button class="paper" class:on={paper === p.paper} role="radio" aria-checked={paper === p.paper} aria-label={p.name} onclick={() => (paper = p.paper)}>
        <SnapPaper paper={p.paper} color={p.paper === PAPER.sticky ? color : null} size={PAPER_SIZE.mini}><span></span></SnapPaper>
      </button>
    {/each}
  </div>

  {#if isSticky}
    <div class="colors" role="radiogroup" aria-label="Sticky colour">
      {#each colors as c (c)}
        <button class="color" class:on={color === c} role="radio" aria-checked={color === c} aria-label="Colour {c}" style:--c={c} onclick={() => (color = c)}></button>
      {/each}
    </div>
  {/if}

  <div class="ink">
    <SnapPaper {paper} color={isSticky ? color : null}>
      <!-- svelte-ignore a11y_autofocus -->
      <textarea use:plainText class="text" aria-label="Snap text" placeholder="Write it down…" rows="4" autofocus={!start} bind:value={text}></textarea>
    </SnapPaper>
  </div>

  <div class="p-field">
    <span class="p-label">stack</span>
    <ChoicePicker value={picked} {choices} what="Stack" field onpick={pickStack} />
  </div>

  <div class="p-field">
    <span class="p-label">tags</span>
    <TagEditor {tags} onchange={(next) => (tags = next)} />
  </div>

  {#if start}
    <div class="acts">
      <button class="act" aria-pressed={pinned} onclick={() => (pinned = !pinned)}><Icon name="pin" size={18} />{pinned ? 'Unpin from top' : 'Pin to top'}</button>
      <button class="act" onclick={cut}><Icon name="cut" size={18} />Cut</button>
      <button class="act danger" onclick={ondelete}><Icon name="trash" size={18} />Delete</button>
    </div>
  {/if}
</Sheet>

<style>
  .papers,
  .colors {
    display: flex;
    gap: 10px;
  }

  .paper {
    flex: 1;
    height: 52px;
    padding: 0;
    border: 2px solid transparent;
    border-radius: var(--radius-md);
    background: transparent;
    /* Light papers stay visible on the sheet. */
    box-shadow: 0 0 0 1px var(--border);
    overflow: hidden;
  }

  .paper.on {
    border-color: var(--c-current);
  }

  .paper :global(.sheet) {
    height: 100%;
    min-height: 0;
    padding: 0;
  }

  .color {
    width: 32px;
    height: 32px;
    padding: 0;
    border: 3px solid transparent;
    border-radius: 50%;
    background: var(--c);
    background-clip: padding-box;
    box-shadow: 0 0 0 1px var(--border);
  }

  .color.on {
    box-shadow: 0 0 0 2px var(--c-current);
  }

  .paper:focus-visible,
  .color:focus-visible {
    outline: none;
    border-color: var(--c-hover);
  }

  /* The text sits on the picked paper, in its own hand. */
  .ink {
    display: flex;
    min-height: 120px;
  }

  .text {
    width: 100%;
    min-height: 96px;
    padding: 0;
    border: 0;
    outline: none;
    resize: none;
    background: transparent;
    font: inherit;
    color: var(--board-ink);
  }

  .acts {
    display: flex;
    justify-content: space-between;
    gap: 8px;
  }

  .act {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 44px;
    padding: 0 8px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    font: var(--text-ui);
    color: var(--text-muted);
  }

  .act[aria-pressed='true'] {
    color: var(--pin-on);
  }

  .act.danger {
    color: var(--danger);
  }

  .act:active,
  .act:focus-visible {
    outline: none;
    background: var(--c-hover);
  }
</style>
