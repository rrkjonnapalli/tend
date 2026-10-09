<script lang="ts">
  // The sync mark in the bottom bar, next to the timer chip: ✓ synced, ↻ syncing, ! failed.
  // Hover shows the last sync; a click opens Settings → Sync. Hidden when the vault isn't connected.
  import Icon from './Icon.svelte';
  import type { IconName } from '#lib/types/icon.type.ts';
  import type { SyncState } from '#lib/types/sync.type.ts';
  import { sync } from '#lib/stores/sync.svelte.ts';
  import { ago } from '#lib/util/dates.ts';
  import { routes } from '#lib/util/routes.ts';

  const ICONS: Record<Exclude<SyncState, 'off'>, IconName> = { running: 'reload', failed: 'alert', synced: 'check' };

  const state = $derived(sync.state);
  const title = $derived(titleOf());

  function titleOf() {
    const s = sync.status;
    if (state === 'running') return 'Syncing…';
    if (s?.error) return `Sync failed: ${s.error.message}`;
    if (state === 'failed') return 'The last sync failed';
    if (!s?.lastRunAt) return 'Sync';
    return `Synced ${ago(s.lastRunAt)}`;
  }
</script>

{#if state !== 'off'}
  <a class="ico" class:failed={state === 'failed'} class:spin={state === 'running'} href={routes.settingsSection('sync')} aria-label={title} {title}>
    <Icon name={ICONS[state]} size={14} stroke={2.25} />
    {#if state === 'running'}<span class="txt">Syncing…</span>{/if}
  </a>
{/if}

<style>
  .ico {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    min-width: 28px;
    height: 26px;
    color: var(--c-muted-2);
    text-decoration: none;
  }

  .ico.spin {
    padding: 0 8px;
  }

  .txt {
    font: 400 12px/16px var(--font-ui);
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
