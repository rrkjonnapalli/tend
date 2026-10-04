<script lang="ts">
  // Markdown shown rendered; the pencil (or a double-click) switches to the editor.
  // ⌘↵ saves, Esc cancels. Pair it with EditControls for the pencil / ✕ / ✓ buttons.
  import MarkdownEditor from './MarkdownEditor.svelte';
  import MarkdownView from './MarkdownView.svelte';

  let {
    value,
    onsave,
    editing = $bindable(false),
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
    if (draft !== value) await onsave(draft);
  }

  export function cancel() {
    editing = false;
  }
</script>

{#if editing}
  <MarkdownEditor
    value={draft}
    onchange={(t) => (draft = t)}
    onsave={save}
    oncancel={cancel}
    {placeholder}
    {minHeight}
    {label}
    autofocus />
{:else}
  <div class="view" style:min-height="{minHeight}px">
    <MarkdownView text={value} {empty} ondblclick={edit} />
  </div>
{/if}

<style>
  .view {
    padding: 6px 0;
    cursor: text;
  }
</style>
