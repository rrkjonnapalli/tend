<script lang="ts">
  // A sheet from the bottom of the phone with a header of Cancel · title · action (MD-Token,
  // MD-Quick, MD-Snap-New). Tapping the veil or Esc cancels.
  import type { Snippet } from 'svelte';
  import { onMount } from 'svelte';
  import { pushLayer } from '#lib/util/layers.ts';

  let {
    title,
    action,
    disabled = false,
    onaction,
    onclose,
    children,
  }: {
    title: string;
    action: string;
    disabled?: boolean;
    onaction: () => void | Promise<void>;
    onclose: () => void;
    children: Snippet;
  } = $props();

  let busy = $state(false);

  onMount(() => pushLayer(onclose));

  async function run() {
    if (disabled || busy) return;
    busy = true;
    try {
      await onaction();
    } finally {
      busy = false;
    }
  }
</script>

<button class="veil" tabindex="-1" aria-hidden="true" onclick={onclose}></button>
<div class="sheet" role="dialog" aria-modal="true" aria-label={title}>
  <span class="grip" aria-hidden="true"></span>
  <header>
    <button class="hb" onclick={onclose}>Cancel</button>
    <h2>{title}</h2>
    <button class="hb act" disabled={disabled || busy} onclick={run}>{action}</button>
  </header>
  {@render children()}
</div>

<style>
  .veil {
    position: fixed;
    inset: 0;
    z-index: 60;
    padding: 0;
    border: 0;
    background: rgba(37, 44, 51, 0.08);
  }

  .sheet {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 61;
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-height: 90vh;
    overflow-y: auto;
    padding: 10px 20px calc(20px + env(safe-area-inset-bottom));
    border-radius: 20px 20px 0 0;
    background: var(--c-page);
    box-shadow: 0 0 0 1px var(--border);
  }

  .grip {
    align-self: center;
    width: 36px;
    height: 4px;
    border-radius: 2px;
    background: var(--c-faint);
  }

  header {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
  }

  h2 {
    margin: 0;
    font: var(--text-section);
    font-size: 17px;
  }

  .hb {
    justify-self: start;
    height: 44px;
    padding: 0;
    border: 0;
    background: transparent;
    font: var(--text-ui);
    font-size: 15px;
    color: var(--text-muted);
  }

  .hb.act {
    justify-self: end;
    color: var(--primary);
  }

  .hb:disabled {
    opacity: 0.45;
  }

  .hb:focus-visible {
    outline: none;
    text-decoration: underline;
  }
</style>
