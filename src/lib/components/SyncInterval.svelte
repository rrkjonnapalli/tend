<script lang="ts">
  // Minutes between automatic syncs, edited in place: pencil to edit, ✓ or Enter saves, ✕ or Esc
  // cancels. 0 turns automatic syncs off.
  import EditControls from './EditControls.svelte';
  import { intervalLimits } from '#lib/config/sync.config.ts';
  import { plural } from '#lib/util/plural.ts';

  let { minutes, onsave }: { minutes: number; onsave: (minutes: number) => Promise<void> } = $props();

  let editing = $state(false);
  /** A number input binds a number, or null while it's empty. */
  let draft = $state<number | null>(null);

  const valid = $derived(isValid(draft));
  const shown = $derived(minutes === 0 ? 'Off' : `Every ${plural(minutes, 'minute')}`);

  function isValid(n: number | null): n is number {
    if (n === null || !Number.isInteger(n)) return false;
    return n >= intervalLimits.min && n <= intervalLimits.max;
  }

  function start() {
    draft = minutes;
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
      class="fld"
      class:bad={!valid}
      type="number"
      min={intervalLimits.min}
      max={intervalLimits.max}
      aria-label="Minutes between syncs, 0 for off"
      bind:value={draft}
      use:focus
      {onkeydown} />
    <span class="unit">minutes · 0 is off</span>
  {:else}
    <span class="val">{shown}</span>
  {/if}
  <EditControls {editing} what="sync interval" onedit={start} oncancel={cancel} onsave={save} />
</div>

<style>
  .line {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 28px;
  }

  .val {
    font: 400 14px/20px var(--font-ui);
  }

  .fld {
    width: 72px;
    height: 28px;
    padding: 0 8px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    outline: none;
    background: var(--c-page);
    color: var(--text);
    font: 400 13px/18px var(--font-mono);
  }

  .fld:focus {
    border-color: var(--primary);
  }

  .fld.bad {
    border-color: var(--danger);
  }

  .unit {
    font: 400 13px/18px var(--font-ui);
    color: var(--c-muted-2);
  }
</style>
