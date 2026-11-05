<script lang="ts">
  // The bar above the phone's keyboard while a note is edited (MD-Note-Edit): heading, bold,
  // italic, list, checkbox, link, code, and one to hide the keyboard. Its buttons never take
  // focus, so the editor keeps its selection and the keyboard stays up.
  import type { MdFormat } from '#lib/util/md-format.ts';

  let { onformat, ondone }: { onformat: (kind: MdFormat) => void; ondone: () => void } = $props();

  // From the mockup (docs/mobile/design/MD-Note-Edit.dc.html): H, B and I are letters, the rest icons.
  const tools: { kind: MdFormat; label: string; glyph?: string; path?: string }[] = [
    { kind: 'heading', label: 'Heading', glyph: 'H' },
    { kind: 'bold', label: 'Bold', glyph: 'B' },
    { kind: 'italic', label: 'Italic', glyph: 'I' },
    { kind: 'list', label: 'List', path: '<path d="M9 6h11M9 12h11M9 18h11M4.5 6h.01M4.5 12h.01M4.5 18h.01"/>' },
    { kind: 'check', label: 'Checkbox', path: '<rect x="4" y="4" width="16" height="16" rx="3"/><path d="M8.5 12l2.5 2.5 4.5-5"/>' },
    {
      kind: 'link',
      label: 'Link',
      path: '<path d="M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7L11.5 7"/><path d="M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1.5-1.5"/>',
    },
    { kind: 'code', label: 'Code', path: '<path d="M9 8l-4 4 4 4M15 8l4 4-4 4"/>' },
  ];

  const keep = (e: PointerEvent) => e.preventDefault();
</script>

<div class="bar" role="toolbar" aria-label="Formatting">
  {#each tools as t (t.kind)}
    <button aria-label={t.label} onpointerdown={keep} onclick={() => onformat(t.kind)}>
      {#if t.glyph}
        <span class="g {t.kind}" aria-hidden="true">{t.glyph}</span>
      {:else}
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">{@html t.path}</svg>
      {/if}
    </button>
  {/each}
  <button aria-label="Hide keyboard" onpointerdown={keep} onclick={ondone}>
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="4" width="18" height="11" rx="2"/><path d="M7 8h.01M11 8h.01M15 8h.01M8 12h8M9 19l3 2 3-2"/></svg>
  </button>
</div>

<style>
  .bar {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 30;
    display: flex;
    justify-content: space-between;
    height: 48px;
    padding: 0 12px;
    border-top: 1px solid var(--border);
    background: var(--c-page);
  }

  button {
    display: grid;
    place-items: center;
    width: 44px;
    height: 48px;
    border: 0;
    background: transparent;
    color: var(--text-muted);
  }

  .g {
    font: 700 18px/1 var(--font-own-ui);
  }

  .g.italic {
    font-style: italic;
  }

  button:active,
  button:focus-visible {
    outline: none;
    background: var(--c-hover);
  }
</style>
