<script lang="ts">
  // The "+ new …" line at the top of a list page (notes, tasks, people). Enter adds; the page
  // decides what adding means. An error shows right under it.
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { focusOnMount } from '#lib/actions/focus.action.ts';

  let {
    value = $bindable(''),
    input = $bindable(),
    label,
    placeholder,
    error = '',
    onadd,
    oninput,
  }: {
    value?: string;
    /** The field, for pages that focus it from a URL flag. */
    input?: HTMLInputElement;
    /** Accessible name, e.g. "New note". */
    label: string;
    placeholder: string;
    error?: string;
    onadd: () => void;
    oninput?: () => void;
  } = $props();

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') onadd();
  }
</script>

<label class="add">
  <span class="plus" aria-hidden="true">+</span>
  <input use:plainText use:focusOnMount data-row-entry bind:this={input} aria-label={label} {placeholder} bind:value {oninput} {onkeydown} />
</label>
{#if error}<span class="err-line under" role="alert">{error}</span>{/if}

<style>
  .add {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: 36px;
    font: var(--text-ui);
    color: var(--c-muted-2);
  }

  .plus {
    width: 16px;
    text-align: center;
  }

  input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: none;
    background: transparent;
    font: var(--text-ui);
    color: var(--text);
  }

  input::placeholder {
    color: var(--c-muted-2);
  }

  /* Lined up with the text, past the plus. */
  .under {
    padding: 0 0 8px 24px;
  }
</style>
