<script lang="ts">
  // A whole-number setting edited in place (the sync interval, the file size limit): pencil to
  // edit, ✓ or Enter saves, ✕ or Esc cancels.
  import EditControls from './EditControls.svelte';

  let {
    value,
    min,
    max,
    shown,
    unit,
    label,
    what,
    onsave,
  }: {
    value: number;
    min: number;
    max: number;
    /** The value as text while not editing: "Every 15 minutes", "30 MB". */
    shown: string;
    /** After the field while editing: "minutes · 0 is off". */
    unit: string;
    /** The field's accessible name. */
    label: string;
    /** Names the setting for the pencil: "sync interval". */
    what: string;
    onsave: (value: number) => Promise<void>;
  } = $props();

  let editing = $state(false);
  /** A number input binds a number, or null while it's empty. */
  let draft = $state<number | null>(null);

  const valid = $derived(isValid(draft));

  function isValid(n: number | null): n is number {
    if (n === null || !Number.isInteger(n)) return false;
    return n >= min && n <= max;
  }

  function start() {
    draft = value;
    editing = true;
  }

  function cancel() {
    editing = false;
  }

  async function save() {
    if (!isValid(draft)) return;
    await onsave(draft);
    editing = false;
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') save();
    if (e.key === 'Escape') {
      e.stopPropagation();
      cancel();
    }
  }

  function focus(node: HTMLInputElement) {
    node.select();
  }
</script>

<div class="line">
  {#if editing}
    <input
      class="text-field mono num"
      class:bad={!valid}
      type="number"
      {min}
      {max}
      aria-label={label}
      bind:value={draft}
      use:focus
      {onkeydown} />
    <span class="unit">{unit}</span>
  {:else}
    <span class="val">{shown}</span>
  {/if}
  <EditControls {editing} {what} onedit={start} oncancel={cancel} onsave={save} />
</div>

<style>
  .line {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 28px;
  }

  .val {
    font: var(--text-ui);
  }

  .num {
    width: 72px;
  }

  .unit {
    font: var(--text-caption);
    color: var(--c-muted-2);
  }
</style>
