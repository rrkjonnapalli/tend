<script lang="ts">
  // The one markdown editor (CodeMirror 6), used by the note page, MarkdownField and comments.
  import { onMount } from 'svelte';
  import { EditorState } from '@codemirror/state';
  import { EditorView, keymap, placeholder as placeholderExt } from '@codemirror/view';
  import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
  import { markdown } from '@codemirror/lang-markdown';
  import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
  import { tags as t } from '@lezer/highlight';

  let {
    value,
    onchange,
    onsave,
    oncancel,
    onblur,
    placeholder = '',
    variant = 'field',
    autofocus = false,
    minHeight = 0,
    label,
  }: {
    value: string;
    onchange: (text: string) => void;
    /** ⌘↵ */
    onsave?: () => void;
    /** Esc */
    oncancel?: () => void;
    onblur?: () => void;
    placeholder?: string;
    /** "note": the full-page editor; "field": descriptions and comments. */
    variant?: 'note' | 'field';
    autofocus?: boolean;
    /** Minimum height of the text area in px. */
    minHeight?: number;
    label: string;
  } = $props();

  let host: HTMLDivElement;
  let view: EditorView | undefined;

  const highlight = HighlightStyle.define([
    { tag: t.heading, fontWeight: '600' },
    { tag: t.strong, fontWeight: '700' },
    { tag: t.emphasis, fontStyle: 'italic' },
    { tag: t.link, color: 'var(--link)' },
    { tag: t.url, color: 'var(--c-muted-2)' },
    { tag: t.monospace, color: 'var(--text-muted)' },
    { tag: t.quote, color: 'var(--text-muted)' },
    { tag: [t.processingInstruction, t.meta], color: 'var(--c-faint)' },
  ]);

  onMount(() => {
    view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: value,
        extensions: [
          history(),
          keymap.of([
            { key: 'Mod-Enter', run: () => (onsave?.(), !!onsave) },
            { key: 'Escape', run: () => (oncancel?.(), !!oncancel) },
            indentWithTab,
            ...defaultKeymap,
            ...historyKeymap,
          ]),
          markdown(),
          syntaxHighlighting(highlight),
          EditorView.lineWrapping,
          placeholderExt(placeholder),
          EditorView.contentAttributes.of({ 'aria-label': label }),
          EditorView.updateListener.of((u) => {
            if (u.docChanged) onchange(u.state.doc.toString());
            if (u.focusChanged && !u.view.hasFocus) onblur?.();
          }),
        ],
      }),
    });
    if (autofocus) {
      view.focus();
      view.dispatch({ selection: { anchor: view.state.doc.length } });
    }
    return () => view?.destroy();
  });

  // Outside changes (a file edited elsewhere) replace the text without losing the editor.
  $effect(() => {
    const next = value;
    if (view && next !== view.state.doc.toString()) {
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: next } });
    }
  });

  export function focus() {
    view?.focus();
  }
</script>

<div class="editor {variant}" style:--min-h="{minHeight}px" bind:this={host}></div>

<style>
  .editor :global(.cm-editor) {
    background: transparent;
  }

  .editor :global(.cm-editor.cm-focused) {
    outline: none;
  }

  .editor :global(.cm-scroller) {
    font-family: var(--font-mono);
  }

  .editor :global(.cm-content) {
    padding: 0;
    min-height: var(--min-h);
    caret-color: var(--text);
  }

  .editor :global(.cm-line) {
    padding: 0;
  }

  .editor :global(.cm-placeholder) {
    color: var(--c-muted-2);
  }

  .editor :global(.cm-selectionBackground),
  .editor :global(.cm-focused .cm-selectionBackground) {
    background: var(--primary-soft) !important;
  }

  .note :global(.cm-scroller) {
    font: 400 15px/26px var(--font-mono);
  }

  .field :global(.cm-scroller) {
    font: 400 14px/22px var(--font-mono);
    max-height: 360px;
    overflow: auto;
  }

  .field :global(.cm-content) {
    padding: 6px 0;
  }
</style>
