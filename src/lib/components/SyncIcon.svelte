<script lang="ts">
  // The sync button in the bottom bar, next to the timer chip. A click syncs now (unless a run is
  // going); it spins with "Syncing…" during a run and turns red after a failure. Hover tells the
  // last sync. Hidden when the vault isn't connected; Settings → Sync holds the rest.
  import Icon from './Icon.svelte';
  import { sync } from '#lib/stores/sync.svelte.ts';
  import { ago } from '#lib/util/dates.ts';

  const state = $derived(sync.state);
  const title = $derived(titleOf());

  function titleOf() {
    const s = sync.status;
    if (state === 'running') return 'Syncing…';
    if (s?.error) return `Sync failed: ${s.error.message}`;
    if (state === 'failed') return 'The last sync failed';
    if (!s?.lastOkAt) return 'Sync now';
    return `Synced ${ago(s.lastOkAt)} · click to sync now`;
  }

  function syncNow() {
    if (state === 'running') return;
    sync.now();
  }
</script>

{#if state !== 'off'}
  <button class="ico" class:failed={state === 'failed'} class:spin={state === 'running'} aria-label={title} {title} onclick={syncNow}>
    <Icon name="sync" size={14} stroke={2} />
    {#if state === 'running'}<span class="txt">Syncing…</span>{/if}
  </button>
{/if}

<style>
  .ico {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    min-width: 28px;
    height: 26px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--c-muted-2);
    cursor: pointer;
  }

  .ico.spin {
    padding: 0 8px;
    cursor: default;
  }

  .txt {
    font: var(--text-meta);
  }

  .ico:hover {
    color: var(--text);
  }

  .ico.failed {
    color: var(--danger);
  }

  /* Only the icon turns, not the text. */
  .spin :global(svg) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .spin :global(svg) {
      animation: none;
    }
  }
</style>
