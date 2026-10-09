<script lang="ts">
  // The fields for reaching a sync remote, shared by Settings → Sync (connect) and the Start page
  // (restore): URL and token, with branch and username under "More", and the public-repo warning.
  import Icon from './Icon.svelte';
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import type { SyncSetup } from '#lib/types/sync.type.ts';
  import { publicRepoWarning } from '#lib/config/sync.config.ts';

  let { setup = $bindable() }: { setup: SyncSetup } = $props();

  let more = $state(false);
</script>

<div class="fields">
  <label class="row">
    <span class="lbl">Repo URL</span>
    <input use:plainText class="fld mono" placeholder="https://github.com/you/notes.git" bind:value={setup.url} />
  </label>
  <label class="row">
    <span class="lbl">Token</span>
    <input class="fld mono" type="password" autocomplete="off" placeholder="saved in the Keychain only" bind:value={setup.token} />
  </label>
  <button class="more" aria-expanded={more} onclick={() => (more = !more)}>
    <Icon name={more ? 'chevronDown' : 'chevronRight'} size={10} stroke={3} />More
  </button>
  {#if more}
    <label class="row">
      <span class="lbl">Branch</span>
      <input use:plainText class="fld mono" placeholder="main" bind:value={setup.branch} />
    </label>
    <label class="row">
      <span class="lbl">Username</span>
      <input use:plainText class="fld mono" placeholder="x-access-token" bind:value={setup.username} />
    </label>
  {/if}
  <p class="warn">{publicRepoWarning}</p>
</div>

<style>
  .fields {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .lbl {
    flex: 0 0 80px;
    font: var(--text-caption);
    color: var(--text-muted);
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
    font: var(--text-mono);
  }

  .fld:focus {
    border-color: var(--primary);
  }

  .more {
    display: flex;
    align-items: center;
    gap: 6px;
    align-self: flex-start;
    padding: 0;
    border: 0;
    background: transparent;
    font: var(--text-caption);
    color: var(--c-muted-2);
    cursor: pointer;
  }

  .more:hover {
    color: var(--text);
  }

  .warn {
    margin: 0;
    font: var(--text-meta);
    color: var(--c-muted-2);
  }
</style>
