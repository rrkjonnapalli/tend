<script lang="ts">
  // Markdown shown rendered; the pencil (or a double-click) switches to the editor.
  // ⌘↵ saves, Esc cancels. Pair it with EditControls for the pencil / ✕ / ✓ buttons.
  import MarkdownEditor from './MarkdownEditor.svelte';
  import MarkdownView from './MarkdownView.svelte';
  import { toAppError } from '#lib/errors/app-error.ts';
  import { errors } from '#lib/errors/registry.ts';

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

  type Saver = (text: string) => void | Promise<void>;

  let draft = $state('');
  /** The edit under way: the text it started from, and the save of the item it started on (a
   *  page may show another item by the time the field goes away). Not state: only read on leave. */
  let open: { from: string; saver: Saver } | null = null;

  export function edit() {
    draft = value;
    open = { from: value, saver: onsave };
    editing = true;
  }

  /** Closes the editor once the text is saved; a failed save keeps it open with the draft. */
  export async function save() {
    if (!editing) return;
    if (draft !== value) await (open?.saver ?? onsave)(draft);
    open = null;
    editing = false;
  }

  export function cancel() {
    open = null;
    editing = false;
  }

  /** Saves an edit left open when the field goes away (another item or page opened). The item may
   *  have been deleted meanwhile: then there is nothing to save to. */
  async function saveOnLeave(edit: { from: string; saver: Saver }, text: string) {
    try {
      await edit.saver(text);
    } catch (e) {
      if (!toAppError(e).is(errors.NOT_FOUND)) throw e;
    }
  }

  $effect(() => () => {
    const changed = open !== null && draft !== open.from;
    if (open && changed) void saveOnLeave(open, draft);
  });
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
