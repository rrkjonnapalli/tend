<script lang="ts">
  // A snap's paper: scrap, slip, lined, plain card or sticky (tinted from its colour). Shared by
  // the board and the editor, which puts its text field inside.
  import type { Snippet } from 'svelte';
  import { stickyColor } from '#lib/config/library.config.ts';
  import { PAPER, type Paper } from '#lib/enums/snap.enum.ts';
  import { stickyTint } from '#lib/util/snaps.ts';

  let {
    paper,
    color,
    editor = false,
    children,
  }: {
    paper: Paper;
    color: string | null;
    /** The editor's fixed-size paper: room for the corner and foot buttons. */
    editor?: boolean;
    children: Snippet;
  } = $props();

  const tint = $derived(paper === PAPER.sticky ? stickyTint(color ?? stickyColor) : null);
</script>

<div
  class="sheet"
  class:scrap={paper === PAPER.scrap}
  class:slip={paper === PAPER.slip}
  class:lined={paper === PAPER.lined}
  class:plain={paper === PAPER.plain}
  class:sticky={paper === PAPER.sticky}
  class:editor
  style:--band={tint?.band}
  style:--tint={tint?.sheet}>
  {@render children()}
</div>

<style>
  .sheet {
    flex: 1 1 auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
    color: var(--board-ink);
    font: 400 14px/24px var(--font-mono);
  }

  .scrap {
    min-height: 150px;
    padding: 28px 20px 30px;
    background: var(--paper);
    clip-path: polygon(0 0, 100% 0, 100% calc(100% - 10px), 94% 100%, 88% calc(100% - 8px), 81% calc(100% - 2px), 74% calc(100% - 9px), 66% 100%, 58% calc(100% - 8px), 50% calc(100% - 2px), 42% calc(100% - 9px), 34% 100%, 26% calc(100% - 8px), 18% calc(100% - 2px), 10% calc(100% - 9px), 4% calc(100% - 1px), 0% calc(100% - 8px));
  }

  .slip {
    padding: 28px 18px 12px;
    background: var(--paper-kraft);
    box-shadow: var(--paper-shadow);
    font: var(--text-code);
  }

  .lined {
    min-height: 170px;
    padding: 28px 20px 14px 44px;
    background-color: var(--paper);
    background-image:
      linear-gradient(90deg, transparent 33px, var(--paper-margin) 33px, var(--paper-margin) 34px, transparent 34px),
      repeating-linear-gradient(transparent 0 23px, var(--paper-rule) 23px 24px);
    background-position: 0 0, 0 4px;
    box-shadow: var(--paper-shadow);
  }

  .plain {
    padding: 28px 20px 14px;
    background: var(--paper);
    box-shadow: var(--paper-shadow);
  }

  .sticky {
    min-height: 220px;
    padding: 28px 20px 14px;
    background: linear-gradient(var(--band) 0 14px, var(--tint) 14px);
    box-shadow: var(--paper-shadow);
    font: 500 23px/28px var(--font-hand);
  }

  .editor {
    min-height: 0;
    padding-top: 50px;
    padding-bottom: 68px;
    padding-right: 18px;
  }

  .editor.lined {
    padding-top: 28px;
  }

  .editor.slip {
    font: 400 14px/20px var(--font-mono);
  }

  .editor:not(.slip, .sticky) {
    font: 400 15px/24px var(--font-mono);
  }

  .editor.sticky {
    font: 500 26px/30px var(--font-hand);
  }
</style>
