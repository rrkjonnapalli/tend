<script lang="ts">
  // A snap's paper: scrap, slip, lined, plain card or sticky (tinted from its colour). Shared by
  // the board, the editor (which puts its text field inside) and Home's mini board.
  import type { Snippet } from 'svelte';
  import { stickyColor } from '#lib/config/library.config.ts';
  import { PAPER, PAPER_SIZE, type Paper, type PaperSize } from '#lib/enums/snap.enum.ts';
  import { stickyTint } from '#lib/util/snaps.ts';

  let {
    paper,
    color,
    size = PAPER_SIZE.board,
    children,
  }: {
    paper: Paper;
    color: string | null;
    /** The editor's paper has room for its corner and foot buttons; Home's is small. */
    size?: PaperSize;
    children: Snippet;
  } = $props();

  const tint = $derived(paper === PAPER.sticky ? stickyTint(color ?? stickyColor) : null);
</script>

<!-- A sticky is a light surface in either theme (tokens.css): bright sheet, dark ink. -->
<div
  class="sheet"
  data-surface={paper === PAPER.sticky ? 'light' : undefined}
  class:scrap={paper === PAPER.scrap}
  class:slip={paper === PAPER.slip}
  class:lined={paper === PAPER.lined}
  class:plain={paper === PAPER.plain}
  class:sticky={paper === PAPER.sticky}
  class:editor={size === PAPER_SIZE.editor}
  class:mini={size === PAPER_SIZE.mini}
  style:--band={tint?.band}
  style:--tint={tint?.sheet}>
  {@render children()}
</div>

<style>
  /* --rule: a lined paper's line pitch; --rule-top: where its first rule starts; --margin: its
     red margin line, from the left; --band-size: a sticky's darker top band. */
  .sheet {
    --rule: 24px;
    --rule-top: 4px;
    --margin: 33px;
    --band-size: 14px;
    flex: 1 1 auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
    color: var(--board-ink);
    font: var(--text-snap);
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
      linear-gradient(90deg, transparent var(--margin), var(--paper-margin) var(--margin), var(--paper-margin) calc(var(--margin) + 1px), transparent calc(var(--margin) + 1px)),
      repeating-linear-gradient(transparent 0 calc(var(--rule) - 1px), var(--paper-rule) calc(var(--rule) - 1px) var(--rule));
    background-position: 0 0, 0 var(--rule-top);
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
    background: linear-gradient(var(--band) 0 var(--band-size), var(--tint) var(--band-size));
    box-shadow: var(--paper-shadow);
    font: var(--text-hand-sticky);
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
    font: var(--text-snap-slip-edit);
  }

  .editor:not(.slip, .sticky) {
    font: var(--text-snap-edit);
  }

  .editor.sticky {
    font: var(--text-hand-sticky-edit);
  }

  /* Home's mini board: every paper smaller, with no least height. */
  .mini {
    --rule: 20px;
    --rule-top: 2px;
    --margin: 21px;
    --band-size: 12px;
    min-height: 0;
    padding: 22px 14px 10px;
    font: var(--text-snap-mini);
  }

  /* Room for the torn edge. */
  .mini.scrap {
    padding-bottom: 22px;
  }

  .mini.lined {
    padding-left: 30px;
  }

  .mini.sticky {
    font: var(--text-hand-sticky-mini);
  }
</style>
