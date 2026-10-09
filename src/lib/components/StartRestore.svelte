<script lang="ts">
  // "Restore from Git" on the Start page: an empty folder, the remote's fields, then Restore.
  // The folder becomes the vault, filled from the repo, and opens.
  import { open } from '@tauri-apps/plugin-dialog';
  import Icon from './Icon.svelte';
  import SyncFields from './SyncFields.svelte';
  import { syncAPI } from '#lib/api/sync.ts';
  import { emptySetup } from '#lib/config/sync.config.ts';
  import { messageOf } from '#lib/errors/app-error.ts';
  import type { Check } from '#lib/types/vault.type.ts';

  let { onrestored }: { onrestored: (path: string) => Promise<void> } = $props();

  let shown = $state(false);
  let path = $state<string | null>(null);
  let setup = $state(emptySetup());
  let busy = $state(false);
  let failure = $state<string | null>(null);

  const ready = $derived(path !== null && setup.url.trim() !== '' && setup.token !== '' && !busy);

  /** Why the folder couldn't become a vault, from its failed check step. */
  function problemOf(check: Check) {
    const failed = check.steps.find((s) => s.status === 'error');
    return failed?.detail ?? 'This folder can’t be used';
  }

  async function choose() {
    const picked = await open({ directory: true, title: 'Choose an empty folder to restore into' });
    if (typeof picked === 'string') path = picked;
  }

  async function restore() {
    if (!ready || !path) return;
    busy = true;
    failure = null;
    try {
      const check = await syncAPI.restore({ path, setup });
      if (check.ok) await onrestored(check.path);
      else failure = problemOf(check);
    } catch (e) {
      failure = messageOf(e);
    } finally {
      busy = false;
    }
  }
</script>

{#if !shown}
  <button class="link-btn lead" onclick={() => (shown = true)}><Icon name="reload" size={15} />Restore from Git…</button>
{:else}
  <div class="box">
    <div class="pick">
      <button class="btn-ghost" onclick={choose}>{path ? 'Change' : 'Choose empty folder…'}</button>
      {#if path}<span class="p">{path}</span>{/if}
    </div>
    <SyncFields bind:setup />
    {#if failure}<div class="banner-err" role="alert">{failure}</div>{/if}
    <div class="row">
      <button class="btn-primary lg" disabled={!ready} onclick={restore}>{busy ? 'Restoring…' : 'Restore'}</button>
      <button class="btn-ghost lg" onclick={() => (shown = false)}>Cancel</button>
    </div>
  </div>
{/if}

<style>
  .box {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .pick {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .p {
    flex: 1;
    min-width: 0;
    font: var(--text-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row {
    display: flex;
    gap: var(--space-2);
  }
</style>
