<script lang="ts">
  // The add row under a colored list in Settings (statuses, labels): a name, and for statuses a
  // value that follows the name (via `toValue`) until it is typed in by hand.
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import Icon from './Icon.svelte';

  let {
    what,
    toValue,
    onadd,
  }: {
    /** The kind of item, for placeholders and messages: "status", "label". */
    what: string;
    /** Turns the name into the item's value; leave out for items without a value. */
    toValue?: (name: string) => string;
    /** Adds the item; returns why it failed, or null. */
    onadd: (item: { name: string; value: string }) => Promise<string | null>;
  } = $props();

  let name = $state('');
  let value = $state('');
  let valueTouched = $state(false);
  let error = $state('');

  /** Why the typed item can't be added, or '' when it can. */
  function problemOf(text: string, val: string): string {
    if (!text) return `Give the ${what} a name.`;
    if (toValue && !val) return `Give the ${what} a value.`;
    return '';
  }

  function reset() {
    name = '';
    value = '';
    valueTouched = false;
  }

  async function add() {
    const text = name.trim();
    const val = toValue ? toValue(value || text) : '';
    error = problemOf(text, val);
    if (error) return;
    const failure = await onadd({ name: text, value: val });
    if (failure) error = failure;
    else reset();
  }

  function onNameInput() {
    if (toValue && !valueTouched) value = toValue(name);
  }

  function onValueInput() {
    valueTouched = true;
    value = value.toUpperCase();
  }

  function addKeys(e: KeyboardEvent) {
    if (e.key === 'Enter') add();
  }
</script>

<div class="addrow">
  <span class="plus">+</span>
  <span></span>
  <input use:plainText aria-label="New {what} name" placeholder="add {what}" bind:value={name} oninput={onNameInput} onkeydown={addKeys} />
  {#if toValue}
    <input use:plainText class="vin" aria-label="New {what} value" placeholder="VALUE" bind:value oninput={onValueInput} onkeydown={addKeys} />
  {:else}
    <span></span>
  {/if}
  <span class="acts">
    {#if name.trim()}<button class="icon-btn sm ok" aria-label="Add {what}" title="Add" onclick={add}><Icon name="check" size={13} stroke={2.25} /></button>{/if}
  </span>
</div>
{#if error}<span class="adderr" role="alert">{error}</span>{/if}

<style>
  .addrow {
    display: grid;
    grid-template-columns: 16px 14px minmax(0, 1fr) 150px 84px;
    align-items: center;
    gap: 12px;
    min-height: 36px;
    font: var(--text-ui);
    color: var(--c-muted-2);
  }

  .plus {
    text-align: center;
  }

  .acts {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 2px;
  }

  .addrow input {
    min-width: 0;
    border: 0;
    outline: none;
    background: transparent;
    font: var(--text-ui);
    color: var(--text);
  }

  .addrow input::placeholder {
    color: var(--c-muted-2);
  }

  .addrow .vin {
    font: var(--text-mono-meta);
    color: var(--text-muted);
    letter-spacing: 0.02em;
    text-align: right;
  }

  .addrow .vin::placeholder {
    color: var(--c-muted-2);
  }

  .adderr {
    padding: 0 0 0 54px;
    font: var(--text-meta);
    color: var(--danger);
  }
</style>
