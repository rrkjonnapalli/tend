<script lang="ts">
  // Replace token (MD-Token): the remote refused the saved token, so automatic sync waits for a new
  // one. Save stores it on this phone and syncs at once.
  import Sheet from './Sheet.svelte';
  import { syncAPI } from '#lib/api/sync.ts';
  import { sync } from '#lib/stores/sync.svelte.ts';
  import { focusOnMount } from '#lib/actions/focus.action.ts';

  let { onclose }: { onclose: () => void } = $props();

  let token = $state('');
  const repo = $derived(sync.status?.config?.url.replace(/^https?:\/\//, '').replace(/\.git$/, '') ?? 'the repo');

  async function save() {
    await syncAPI.setToken(token);
    await sync.load();
    await sync.now();
    onclose();
  }
</script>

<Sheet title="Replace token" action="Save" disabled={token === ''} onaction={save} {onclose}>
  <p class="why">The remote turned down the saved token, so sync has stopped. Your changes stay on this phone until you add a new one.</p>
  <label class="f">
    <span class="lbl">New token</span>
    <input use:focusOnMount class="in" type="password" autocomplete="off" autocapitalize="off" bind:value={token} />
  </label>
  <p class="hint">Needs read and write access to {repo}.</p>
</Sheet>

<style>
  .why {
    margin: 0;
    font: var(--text-ui);
    font-size: 15px;
    line-height: 22px;
    text-align: center;
    color: var(--text-muted);
  }

  .f {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .lbl,
  .hint {
    margin: 0;
    font: var(--text-caption);
    color: var(--text-muted);
  }

  .in {
    height: 44px;
    padding: 0 12px;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    background: var(--c-chip);
    font: var(--text-mono);
    font-size: 15px;
    color: var(--text);
  }

  .in:focus {
    outline: none;
    border-color: var(--primary);
  }
</style>
