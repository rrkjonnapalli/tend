<script lang="ts">
  // Settings → Sync. Not connected: the remote's fields and Connect. Connected: the remote, the
  // token (kept in the Keychain only), the schedule, the last run, Sync now and Disconnect.
  import ConfirmDialog from './ConfirmDialog.svelte';
  import SyncFields from './SyncFields.svelte';
  import NumberSetting from './NumberSetting.svelte';
  import { intervalLimits } from '#lib/config/sync.config.ts';
  import { plural } from '#lib/util/plural.ts';
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { emptySetup, publicRepoWarning } from '#lib/config/sync.config.ts';
  import { messageOf } from '#lib/errors/app-error.ts';
  import { sync } from '#lib/stores/sync.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { timestamp } from '#lib/util/dates.ts';

  let setup = $state(emptySetup());
  let busy = $state(false);
  let failure = $state<string | null>(null);
  let replacing = $state(false);
  let token = $state('');
  let confirmDisconnect = $state(false);

  const status = $derived(sync.status);
  const config = $derived(status?.config ?? null);
  const canConnect = $derived(setup.url.trim() !== '' && setup.token !== '' && !busy);
  const synced = $derived(status?.lastOkAt ? `${timestamp(status.lastOkAt)} · synced` : 'Not yet');
  const failed = $derived(failedText());

  /** When the last run failed and why; only shown while that failure is the latest run. */
  function failedText() {
    const at = status?.failedAt ?? status?.lastRunAt;
    if (!at) return '';
    const why = status?.error?.message ?? `failed (${status?.lastResult})`;
    return `${timestamp(at)} · ${why}`;
  }

  async function connect() {
    if (!canConnect) return;
    busy = true;
    failure = null;
    try {
      await sync.connect(setup);
      setup = emptySetup();
      toast.show('Connected: the vault is in the repo');
    } catch (e) {
      failure = messageOf(e);
    } finally {
      busy = false;
    }
  }

  async function saveToken() {
    if (!token) return;
    await sync.setToken(token);
    token = '';
    replacing = false;
  }

  async function disconnect() {
    await sync.disconnect();
    confirmDisconnect = false;
  }
</script>

{#if !config}
  <section class="group">
    <h2 class="section-title">Connect a repo</h2>
    <span class="cap">Use a new Git repo (a README in it is fine). Tend pushes this vault there, then syncs it on its own.</span>
    <SyncFields bind:setup />
    {#if failure}<div class="banner-err" role="alert">{failure}</div>{/if}
    <div><button class="btn-primary" disabled={!canConnect} onclick={connect}>{busy ? 'Connecting…' : 'Connect'}</button></div>
  </section>
{:else}
  <section class="group">
    <h2 class="section-title">Repo</h2>
    <div class="kv"><span class="k">URL</span><span class="v mono">{config.url}</span></div>
    <div class="kv"><span class="k">Branch</span><span class="v mono">{config.branch}</span></div>
    <div class="kv">
      <span class="k">Token</span>
      {#if replacing}
        <input class="text-field mono grow" type="password" autocomplete="off" aria-label="New token" bind:value={token} use:plainText />
        <button class="link-btn" onclick={saveToken}>Save</button>
        <button class="link-btn" onclick={() => (replacing = false)}>Cancel</button>
      {:else}
        <span class="v">{status?.hasToken ? 'Saved in the Keychain' : 'None saved'}</span>
        <button class="link-btn" onclick={() => (replacing = true)}>{status?.hasToken ? 'Replace' : 'Add'}</button>
      {/if}
    </div>
    <p class="warn">{publicRepoWarning}</p>
  </section>

  <section class="group">
    <h2 class="section-title">Schedule</h2>
    <span class="cap">Minutes between automatic syncs. Tend also syncs when the vault opens and when you quit.</span>
    <NumberSetting
      value={config.intervalMinutes}
      min={intervalLimits.min}
      max={intervalLimits.max}
      shown={config.intervalMinutes === 0 ? 'Off' : `Every ${plural(config.intervalMinutes, 'minute')}`}
      unit="minutes · 0 is off"
      label="Minutes between syncs, 0 for off"
      what="sync interval"
      onsave={sync.setInterval} />
  </section>

  <section class="group">
    <h2 class="section-title">Last sync</h2>
    {#if sync.state === 'running'}
      <div class="kv"><span class="v">Syncing…</span></div>
    {:else if sync.state === 'failed'}
      <div class="kv"><span class="k">Last synced</span><span class="v">{synced}</span></div>
      <div class="kv"><span class="k">Last failed</span><span class="v bad">{failed}</span></div>
    {:else}
      <div class="kv"><span class="v">{synced}</span></div>
    {/if}
    <div class="row">
      <button class="btn-ghost" disabled={sync.state === 'running'} onclick={sync.now}>Sync now</button>
      <button class="btn-ghost" onclick={() => (confirmDisconnect = true)}>Disconnect…</button>
    </div>
  </section>
{/if}

{#if confirmDisconnect}
  <ConfirmDialog
    title="Disconnect sync?"
    body="This vault stops syncing and its token is removed from the Keychain. The Git history Tend kept in this folder goes to the Trash; your notes and tasks stay as they are, and so does the remote repo. Connecting again needs a repo with no vault in it."
    confirm="Disconnect"
    danger
    onconfirm={disconnect}
    onclose={() => (confirmDisconnect = false)} />
{/if}

<style>
  .group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .cap,
  .warn {
    margin: 0;
    font: var(--text-caption);
    color: var(--c-muted-2);
  }

  .warn {
    font: var(--text-meta);
  }

  .kv {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 28px;
  }

  .k {
    flex: 0 0 88px;
    font: var(--text-caption);
    color: var(--text-muted);
  }

  .v {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: var(--text-ui);
  }

  .v.mono {
    font: var(--text-mono);
  }

  .v.bad {
    color: var(--danger);
  }

  .row {
    display: flex;
    gap: var(--space-2);
  }
</style>
