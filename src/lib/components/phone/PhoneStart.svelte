<script lang="ts">
  // A phone's first run (MD-Restore, MD-Ready): the repo the computer syncs to, its token, then
  // Restore. The vault goes in Tend's own storage, in the folder named after the repo; if that
  // folder is already there, a sheet asks before replacing it. Spec: tend-mobile-spec.md §10.
  import Icon from '../Icon.svelte';
  import ActionSheet from './ActionSheet.svelte';
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { syncAPI } from '#lib/api/sync.ts';
  import { emptySetup, publicRepoWarning } from '#lib/config/sync.config.ts';
  import { errors } from '#lib/errors/registry.ts';
  import { messageOf, toAppError } from '#lib/errors/app-error.ts';
  import type { Check } from '#lib/types/vault.type.ts';

  let { onrestored }: { onrestored: (path: string) => Promise<void> } = $props();

  let setup = $state(emptySetup());
  let showToken = $state(false);
  let more = $state(false);
  let busy = $state(false);
  let askReplace = $state(false);
  let failure = $state<string | null>(null);

  const ready = $derived(setup.url.trim() !== '' && setup.token !== '' && !busy);

  function problemOf(check: Check) {
    const failed = check.steps.find((s) => s.status === 'error');
    return failed?.detail ?? 'This vault can’t be opened';
  }

  async function restore(replace = false) {
    if (!ready && !replace) return;
    askReplace = false;
    busy = true;
    failure = null;
    try {
      const check = await syncAPI.restorePhone({ setup, replace });
      if (check.ok) await onrestored(check.path);
      else failure = problemOf(check);
    } catch (e) {
      const err = toAppError(e);
      if (err.is(errors.RESTORE_NOT_EMPTY)) askReplace = true;
      else failure = messageOf(e);
    } finally {
      busy = false;
    }
  }
</script>

{#if busy}
  <!-- shortcut: no step counts or progress events from Rust yet; add them with restore progress. -->
  <main class="ready" aria-busy="true">
    <h1>Getting your vault ready…</h1>
    <p class="sub">The first restore brings everything over once. After that, only changes travel.</p>
    <div class="bar" role="progressbar" aria-label="Restoring"><span></span></div>
    <p class="sub">Keep Tend open until it’s done.</p>
  </main>
{:else}
  <main class="start">
    <h1 class="brand">tend<span>.</span></h1>
    <p class="sub">Tend restores your vault from its Git repo onto this phone. Connect the repo your computer syncs to.</p>

    <label class="f">
      <span class="lbl">Repo URL</span>
      <textarea use:plainText class="in mono" rows="2" placeholder="https://github.com/you/notes" autocapitalize="off" spellcheck="false" bind:value={setup.url}></textarea>
    </label>
    <label class="f">
      <span class="lbl">Token</span>
      <span class="in row">
        <input class="mono grow" type={showToken ? 'text' : 'password'} autocomplete="off" autocapitalize="off" bind:value={setup.token} />
        <button class="eye" type="button" aria-label={showToken ? 'Hide token' : 'Show token'} onclick={() => (showToken = !showToken)}>
          <Icon name="eye" size={18} />
        </button>
      </span>
    </label>

    <button class="more" type="button" aria-expanded={more} onclick={() => (more = !more)}>
      <Icon name={more ? 'chevronDown' : 'chevronRight'} size={12} stroke={2.5} />More
      <span class="meta">· branch {setup.branch || 'main'}</span>
    </button>
    {#if more}
      <label class="f">
        <span class="lbl">Branch</span>
        <input use:plainText class="in mono" placeholder="main" autocapitalize="off" bind:value={setup.branch} />
      </label>
      <label class="f">
        <span class="lbl">Username</span>
        <input use:plainText class="in mono" placeholder="x-access-token" autocapitalize="off" bind:value={setup.username} />
      </label>
    {/if}

    <p class="warn">{publicRepoWarning}</p>
    {#if failure}<div class="banner-err" role="alert">{failure}</div>{/if}

    <div class="foot">
      <button class="go" disabled={!ready} onclick={() => restore()}>Restore</button>
      <p class="sub center">No vault yet? Start one on your computer, connect it to a repo, then restore it here.</p>
    </div>
  </main>
{/if}

{#if askReplace}
  <ActionSheet
    title="Replace the vault on this phone?"
    body="This repo is already on this phone. Replacing it deletes that copy and restores the repo again; anything not yet synced from this phone is lost."
    confirm="Replace and restore"
    danger
    onconfirm={() => restore(true)}
    onclose={() => (askReplace = false)} />
{/if}

<style>
  .start,
  .ready {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 72px 20px 24px;
    overflow-y: auto;
  }

  .ready {
    justify-content: center;
    padding: 24px 32px;
  }

  h1 {
    margin: 0;
    font: var(--text-heading);
  }

  .brand {
    font: var(--text-brand);
    font-size: 40px;
    line-height: 48px;
  }

  .brand span {
    color: var(--accent);
  }

  .sub {
    margin: 0;
    font: var(--text-ui);
    font-size: 15px;
    line-height: 22px;
    color: var(--text-muted);
  }

  .center {
    text-align: center;
  }

  .f {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .lbl {
    font: var(--text-caption);
    color: var(--text-muted);
  }

  .in {
    min-height: 44px;
    padding: 10px 12px;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    background: var(--c-chip);
    font: var(--text-mono);
    font-size: 15px;
    color: var(--text);
    resize: none;
  }

  .in:focus,
  .in:focus-within {
    outline: none;
    border-color: var(--primary);
  }

  .row {
    display: flex;
    align-items: center;
    padding: 0 0 0 12px;
  }

  .row input {
    min-width: 0;
    height: 44px;
    border: 0;
    background: transparent;
    font: inherit;
    color: inherit;
  }

  .row input:focus {
    outline: none;
  }

  .grow {
    flex: 1;
  }

  .eye {
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    border: 0;
    background: transparent;
    color: var(--c-muted-2);
  }

  .more {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 44px;
    padding: 0;
    border: 0;
    background: transparent;
    font: var(--text-ui);
    font-size: 15px;
    color: var(--text);
  }

  .meta {
    color: var(--c-muted-2);
  }

  .warn {
    margin: 0;
    font: var(--text-caption);
    color: var(--text-muted);
  }

  .foot {
    margin-top: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .go {
    height: 48px;
    border: 0;
    border-radius: var(--radius-md);
    background: var(--primary);
    color: var(--on-primary);
    font: var(--text-ui-strong);
    font-size: 16px;
  }

  .go:disabled {
    opacity: 0.5;
  }

  .go:focus-visible {
    outline: none;
    background: var(--primary-hover);
  }

  .bar {
    position: relative;
    height: 4px;
    overflow: hidden;
    border-radius: 2px;
    background: var(--c-chip);
  }

  .bar span {
    position: absolute;
    inset: 0 auto 0 0;
    width: 40%;
    border-radius: 2px;
    background: var(--c-current);
    animation: slide 1.4s ease-in-out infinite;
  }

  @keyframes slide {
    from {
      left: -40%;
    }
    to {
      left: 100%;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .bar span {
      animation: none;
      width: 100%;
    }
  }
</style>
