<script lang="ts">
  // Rendered markdown. "note" is the note page look (mono body, large headings); "ui" is the
  // compact look used for descriptions, person notes and comments.
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { renderMarkdown } from '#lib/util/markdown.ts';

  let {
    text,
    variant = 'ui',
    empty = '',
    ondblclick,
  }: { text: string; variant?: 'note' | 'ui'; empty?: string; ondblclick?: () => void } = $props();

  const EXTERNAL_LINK = /^(https?|mailto):/i;

  const html = $derived(renderMarkdown(text));

  // Links open in the default browser instead of replacing the app.
  function onclick(e: MouseEvent) {
    const link = (e.target as HTMLElement).closest('a');
    if (!link) return;
    e.preventDefault();
    const href = link.getAttribute('href') ?? '';
    if (EXTERNAL_LINK.test(href)) openUrl(href);
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="md {variant}" {onclick} {ondblclick}>
  {#if text.trim()}
    {@html html}
  {:else if empty}
    <span class="empty">{empty}</span>
  {/if}
</div>

<style>
  .md {
    user-select: text;
    overflow-wrap: anywhere;
  }

  .md :global(> :first-child) {
    margin-top: 0;
  }

  .md :global(a) {
    color: var(--link);
  }

  .md :global(strong) {
    font-weight: 700;
  }

  .md :global(hr) {
    border: 0;
    height: 1px;
    background: var(--border);
    margin: 20px 0;
  }

  .md :global(pre) {
    margin: 4px 0 14px;
    padding: 10px 12px;
    background: var(--c-hover);
    border-radius: var(--radius-sm);
    font: 400 13px/20px var(--font-mono);
    white-space: pre-wrap;
  }

  .md :global(pre code) {
    background: none;
    padding: 0;
    font: inherit;
  }

  .md :global(code) {
    font: 400 13px/20px var(--font-mono);
    background: var(--c-hover);
    padding: 0 4px;
    border-radius: var(--radius-xs);
  }

  .md :global(blockquote) {
    margin: 0 0 14px;
    padding-left: 14px;
    box-shadow: inset 2px 0 0 var(--border);
    color: var(--text-muted);
  }

  .md :global(blockquote p) {
    margin: 0;
  }

  .md :global(ul),
  .md :global(ol) {
    margin: 0 0 14px;
    padding-left: 22px;
  }

  .md :global(li:has(> input[type='checkbox'])) {
    list-style: none;
    margin-left: -22px;
  }

  .md :global(input[type='checkbox']) {
    appearance: none;
    width: 11px;
    height: 11px;
    margin: 0 10px 0 0;
    border: 1px solid var(--c-muted-2);
    border-radius: 2px;
    vertical-align: 0;
  }

  .md :global(input[type='checkbox']:checked) {
    background: var(--c-muted-2);
    box-shadow: inset 0 0 0 2px var(--c-page);
  }

  .md :global(table) {
    border-collapse: collapse;
    margin: 0 0 14px;
  }

  .md :global(th),
  .md :global(td) {
    border: 1px solid var(--border);
    padding: 4px 8px;
  }

  .empty {
    color: var(--c-muted-2);
  }

  /* Note page */
  .note {
    font: 400 15px/26px var(--font-mono);
  }

  .note :global(h1) {
    font: var(--text-heading);
    margin: 0 0 12px;
  }

  .note :global(h2),
  .note :global(h3),
  .note :global(h4) {
    font: 500 18px/26px var(--font-heading);
    margin: 20px 0 8px;
  }

  .note :global(p) {
    margin: 0 0 14px;
  }

  /* Descriptions, person notes, comments */
  .ui {
    font: 400 14px/22px var(--font-ui);
  }

  .ui :global(h1),
  .ui :global(h2),
  .ui :global(h3),
  .ui :global(h4) {
    font: 500 16px/24px var(--font-heading);
    margin: 2px 0 6px;
  }

  .ui :global(p) {
    margin: 0 0 6px;
  }

  .ui :global(ul),
  .ui :global(ol),
  .ui :global(blockquote) {
    margin-bottom: 6px;
  }

  .ui :global(pre) {
    margin: 4px 0 8px;
    padding: 8px 10px;
  }
</style>
