<script lang="ts">
  // What the window shows before the app: "Opening vault…" while launch waits (for example on
  // macOS asking for folder access), or why the remembered vault couldn't be opened.
  import { vault } from '#lib/stores/vault.svelte.ts';
  import { nav } from '#lib/stores/history.svelte.ts';
  import { routes } from '#lib/util/routes.ts';
  import { errors } from '#lib/errors/registry.ts';

  /** Fast launches skip the loading line entirely. */
  const SHOW_AFTER_MS = 300;
  let slow = $state(false);
  $effect(() => {
    const timer = setTimeout(() => {
      slow = true;
    }, SHOW_AFTER_MS);
    return () => clearTimeout(timer);
  });

  /** A folder-access error gets the System Settings hint. */
  const permission = $derived(vault.problem?.error.is(errors.NO_PERMISSION) ?? false);

  function chooseAnother() {
    vault.dismiss();
    nav.replace(routes.start);
  }
</script>

<main class="gate">
  <div class="box">
    <div class="mark">tend<i>.</i></div>
    {#if vault.problem}
      <h3 class="section-title">Can’t open your vault</h3>
      {#if vault.problem.path}<span class="mono">{vault.problem.path}</span>{/if}
      <div class="banner-err" role="alert">{vault.problem.error.message}</div>
      {#if permission}
        <span class="cap">Allow Tend in System Settings → Privacy &amp; Security → Files and Folders, then try again.</span>
      {/if}
      <div class="acts">
        <button class="btn-primary" onclick={vault.restore}>Try again</button>
        <button class="btn-ghost" onclick={chooseAnother}>Choose another folder</button>
      </div>
    {:else if slow}
      <div class="wait" role="status"><span class="spin"></span>Opening vault…</div>
      <span class="cap">If macOS asks, allow Tend to access the folder.</span>
    {/if}
  </div>
</main>

<style>
  .gate {
    flex: 1;
    display: grid;
    place-items: center;
  }

  .box {
    width: 440px;
    max-width: calc(100% - 48px);
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .mark {
    font: var(--text-brand);
    letter-spacing: -0.01em;
    margin-bottom: 8px;
  }

  .mark i {
    font-style: normal;
    color: var(--accent);
  }

  .mono {
    font: var(--text-mono);
    color: var(--text-muted);
    overflow-wrap: anywhere;
  }

  .cap {
    font: var(--text-caption);
    color: var(--c-muted-2);
  }

  .acts {
    display: flex;
    gap: 8px;
    margin-top: 4px;
  }

  .wait {
    display: flex;
    align-items: center;
    gap: 10px;
    font: var(--text-ui);
    color: var(--text-muted);
  }

  .spin {
    width: 12px;
    height: 12px;
    border: 2px solid var(--border);
    border-top-color: var(--primary);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
