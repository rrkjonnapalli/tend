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
  <input use:plainText use:focusAndSelect class="inline-field field" class:bad={problem} data-no-drag aria-label={label} bind:value={text} {onkeydown} {onblur} />
  {#if problem}<span class="err-line err" role="alert">{problem}</span>{/if}
</span>

<style>
  .wrap {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
  }

  .field {
    flex: 1;
  }

  .err {
    flex: 0 0 auto;
  }
</style>
