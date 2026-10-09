<script lang="ts">
  // Settings → Sync. Not connected: the remote's fields and Connect. Connected: the remote, the
  // token (kept in the Keychain only), the schedule, the last run, Sync now and Disconnect.
  import ConfirmDialog from './ConfirmDialog.svelte';
  import SyncFields from './SyncFields.svelte';
  import SyncInterval from './SyncInterval.svelte';
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { emptySetup, publicRepoWarning } from '#lib/config/sync.config.ts';
  import { messageOf } from '#lib/errors/app-error.ts';
  import { sync } from '#lib/stores/sync.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { ago } from '#lib/util/dates.ts';

  let setup = $state(emptySetup());
  let busy = $state(false);
  let failure = $state<string | null>(null);
  let replacing = $state(false);
  let token = $state('');
  let confirmDisconnect = $state(false);

  const status = $derived(sync.status);
  const config = $derived(status?.config ?? null);
  const canConnect = $derived(setup.url.trim() !== '' && setup.token !== '' && !busy);
  const lastRun = $derived(lastRunText());

  function lastRunText() {
    if (!status?.lastRunAt) return 'Not yet';
    const when = ago(status.lastRunAt);
    if (status.error) return `${when} · ${status.error.message}`;
    if (status.lastResult && status.lastResult !== 'ok') return `${when} · failed (${status.lastResult})`;
    return `${when} · synced`;
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
    <h2 class="h2">Connect a repo</h2>
    <span class="cap">Use a new Git repo (a README in it is fine). Tend pushes this vault there, then syncs it on its own.</span>
    <SyncFields bind:setup />
    {#if failure}<div class="err" role="alert">{failure}</div>{/if}
    <div><button class="pbtn" disabled={!canConnect} onclick={connect}>{busy ? 'Connecting…' : 'Connect'}</button></div>
  </section>
{:else}
  <section class="group">
    <h2 class="h2">Repo</h2>
    <div class="kv"><span class="k">URL</span><span class="v mono">{config.url}</span></div>
    <div class="kv"><span class="k">Branch</span><span class="v mono">{config.branch}</span></div>
    <div class="kv">
      <span class="k">Token</span>
      {#if replacing}
        <input class="fld mono" type="password" autocomplete="off" aria-label="New token" bind:value={token} use:plainText />
        <button class="tlink" onclick={saveToken}>Save</button>
        <button class="tlink" onclick={() => (replacing = false)}>Cancel</button>
      {:else}
        <span class="v">{status?.hasToken ? 'Saved in the Keychain' : 'None saved'}</span>
        <button class="tlink" onclick={() => (replacing = true)}>{status?.hasToken ? 'Replace' : 'Add'}</button>
      {/if}
    </div>
    <p class="warn">{publicRepoWarning}</p>
  </section>

  <section class="group">
    <h2 class="h2">Schedule</h2>
    <span class="cap">Minutes between automatic syncs. Tend also syncs when the vault opens and when you quit.</span>
    <SyncInterval minutes={config.intervalMinutes} onsave={sync.setInterval} />
  </section>

  <section class="group">
    <h2 class="h2">Last sync</h2>
    <div class="kv"><span class="v" class:bad={sync.state === 'failed'}>{status?.running ? 'Syncing…' : lastRun}</span></div>
    <div class="row">
      <button class="gbtn" disabled={status?.running} onclick={sync.now}>Sync now</button>
      <button class="gbtn" onclick={() => (confirmDisconnect = true)}>Disconnect…</button>
    </div>
  </section>
{/if}

{#if confirmDisconnect}
  <ConfirmDialog
    title="Disconnect sync?"
    body="This vault stops syncing and its token is removed from the Keychain. Your notes, tasks and the repo stay as they are. Connecting again needs a repo with no vault in it."
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

  .h2 {
    font: 500 15px/20px var(--font-heading);
    margin: 0;
  }

  .cap,
  .warn {
    margin: 0;
    font: 400 13px/18px var(--font-ui);
    color: var(--c-muted-2);
  }

  .warn {
    font-size: 12px;
    line-height: 16px;
  }

  .kv {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 28px;
  }

  .k {
    flex: 0 0 80px;
    font: 400 13px/18px var(--font-ui);
    color: var(--text-muted);
  }

  .v {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: 400 14px/20px var(--font-ui);
  }

  .v.mono {
    font: 400 13px/18px var(--font-mono);
  }

  .v.bad {
    color: var(--danger);
  }

  .fld {
    flex: 1;
    min-width: 0;
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

  .row {
    display: flex;
    gap: var(--space-2);
  }

  .err {
    padding: 10px 12px;
    border-radius: 6px;
    background: var(--danger-soft);
    color: var(--on-danger-soft);
    font: 400 13px/18px var(--font-ui);
  }

  .tlink {
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--link);
    font: 400 14px/20px var(--font-ui);
    white-space: nowrap;
    cursor: pointer;
  }

  .tlink:hover {
    color: var(--primary-hover);
    text-decoration: underline;
  }

  .pbtn {
    height: 32px;
    padding: 0 16px;
    border: 0;
    border-radius: var(--radius-md);
    background: var(--primary);
    color: var(--on-primary);
    font: 700 14px/20px var(--font-ui);
    cursor: pointer;
  }

  .pbtn:hover:not(:disabled) {
    background: var(--primary-hover);
  }

  .gbtn {
    height: 28px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--c-page);
    color: var(--text);
    font: 400 13px/18px var(--font-ui);
    cursor: pointer;
  }

  .gbtn:hover:not(:disabled) {
    border-color: var(--c-muted-2);
  }

  button:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
