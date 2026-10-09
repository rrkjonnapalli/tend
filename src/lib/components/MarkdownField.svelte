<script lang="ts">
  // Markdown shown rendered; the pencil (or a double-click) switches to the editor.
  // ⌘↵ saves, Esc cancels. Pair it with EditControls for the pencil / ✕ / ✓ buttons.
  import MarkdownEditor from './MarkdownEditor.svelte';
  import MarkdownView from './MarkdownView.svelte';

  let {
    value,
    onsave,
    // No fallback: a parent may bind a value that starts out undefined (one entry per comment).
    editing = $bindable(),
    placeholder = '',
    empty = '',
    minHeight = 120,
    label,
  }: {
    value: string;
    onsave: (text: string) => void | Promise<void>;
    editing?: boolean;
    placeholder?: string;
    /** Shown when there is nothing to render. */
    empty?: string;
    /** Minimum height of the editor in px. */
    minHeight?: number;
    label: string;
  } = $props();

  let draft = $state('');

  export function edit() {
    draft = value;
    editing = true;
  }

  export async function save() {
    if (!editing) return;
    editing = false;
    if (draft === value) return;
    await onsave(draft);
  }

  export function cancel() {
    editing = false;
  }
</script>

{#if editing}
  <div data-zone="field">
    <MarkdownEditor
      value={draft}
      onchange={(text) => (draft = text)}
      onsave={save}
      oncancel={cancel}
      {placeholder}
      {minHeight}
      {label}
      autofocus />
  </div>
{:else}
  <div class="view">
    <MarkdownView text={value} {empty} ondblclick={edit} />
  </div>
{/if}

<style>
  .view {
    padding: 6px 0;
    cursor: text;
  }
</style>
