<script lang="ts">
  // A list row's title being edited in place: Enter or leaving the field saves, Esc cancels.
  // `check` returns why the text can't be saved (shown under the field), or ''.
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { focusAndSelect } from '#lib/actions/focus.action.ts';
  import { untrack } from 'svelte';

  let {
    value,
    label,
    check = () => '',
    onsave,
    oncancel,
  }: {
    value: string;
    /** Accessible name, e.g. "Task title". */
    label: string;
    check?: (text: string) => string;
    /** Called only with changed, valid text. */
    onsave: (text: string) => void;
    oncancel: () => void;
  } = $props();

  // The field starts from the title at the moment editing began.
  let text = $state(untrack(() => value));
  const trimmed = $derived(text.trim());
  const problem = $derived(check(trimmed));

  function save() {
    if (problem) return;
    if (trimmed === value) {
      oncancel();
      return;
    }
    onsave(trimmed);
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      save();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      oncancel();
    }
  }

  /** Leaving the field saves when it can; otherwise the edit is dropped. */
  function onblur() {
    if (problem) {
      oncancel();
      return;
    }
    save();
  }
</script>

<span class="wrap">
  <input use:plainText use:focusAndSelect class="field" class:bad={problem} data-no-drag aria-label={label} bind:value={text} {onkeydown} {onblur} />
  {#if problem}<span class="err">{problem}</span>{/if}
</span>

<style>
  .wrap {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
  }

  /* Same as the inline edits on the person page: a soft fill, no border. */
  .field {
    flex: 1;
    min-width: 0;
    height: 28px;
    padding: 0 8px;
    margin-left: -8px;
    border: 0;
    border-radius: var(--radius-sm);
    outline: none;
    background: var(--c-hover);
    font: inherit;
    color: var(--text);
  }

  .field.bad {
    color: var(--danger);
  }

  .err {
    flex: 0 0 auto;
    font: 400 12px/16px var(--font-ui);
    color: var(--danger);
  }
</style>
