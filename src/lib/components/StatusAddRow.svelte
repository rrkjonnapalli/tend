<script lang="ts">
  // The add row under the statuses: a label, and a value that follows the label as UPPER_SNAKE
  // until it is typed in by hand.
  import Icon from './Icon.svelte';
  import { statusesAPI } from '#lib/api/statuses.ts';
  import { freeSwatch, statusValue } from '#lib/util/statuses.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';

  /** `apply` runs a change and reloads the statuses; it returns why it failed, or null. */
  let { apply }: { apply: (action: () => Promise<unknown>) => Promise<string | null> } = $props();

  let label = $state('');
  let value = $state('');
  let valueTouched = $state(false);
  let error = $state('');

  /** Why the typed status can't be added, or '' when it can. */
  function problemOf(text: string, val: string): string {
    if (!text) return 'Give the status a label.';
    if (!val) return 'Give the status a value.';
    return '';
  }

  function reset() {
    label = '';
    value = '';
    valueTouched = false;
  }

  async function add() {
    const text = label.trim();
    const val = statusValue(value || text);
    error = problemOf(text, val);
    if (error) return;
    const color = freeSwatch(tasks.statuses.map((s) => s.color));
    const failure = await apply(() => statusesAPI.add({ label: text, value: val, color }));
    if (failure) error = failure;
    else reset();
  }

  function onLabelInput() {
    if (!valueTouched) value = statusValue(label);
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
  <input aria-label="New status label" placeholder="add status" bind:value={label} oninput={onLabelInput} onkeydown={addKeys} />
  <input class="vin" aria-label="New status value" placeholder="VALUE" bind:value oninput={onValueInput} onkeydown={addKeys} />
  <span class="acts">
    {#if label.trim()}<button class="icon-btn sm ok" aria-label="Add status" title="Add" onclick={add}><Icon name="check" size={13} stroke={2.25} /></button>{/if}
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
    font: 400 14px/20px var(--font-ui);
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
    font: 400 14px/20px var(--font-ui);
    color: var(--text);
  }

  .addrow input::placeholder {
    color: var(--c-muted-2);
  }

  .addrow .vin {
    font: 400 12px/16px var(--font-mono);
    color: var(--text-muted);
    letter-spacing: 0.02em;
  }

  .addrow .vin::placeholder {
    color: var(--c-faint);
  }

  .adderr {
    padding: 0 0 0 54px;
    font: 400 12px/16px var(--font-ui);
    color: var(--danger);
  }
</style>
