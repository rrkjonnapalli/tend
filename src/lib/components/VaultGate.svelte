<script lang="ts">
  // What the window shows before the app: "Opening vault…" while launch waits (for example on
  // macOS asking for folder access), or why the remembered vault couldn't be opened.
  import { vault } from '#lib/stores/vault.svelte.ts';
  import { nav } from '#lib/stores/history.svelte.ts';

  /** Fast launches skip the loading line entirely. */
  const SHOW_AFTER_MS = 300;
  let slow = $state(false);
  $effect(() => {
    const t = setTimeout(() => (slow = true), SHOW_AFTER_MS);
    return () => clearTimeout(t);
  });

  const permission = $derived(vault.problem?.message.includes('permission') ?? false);

  function chooseAnother() {
    vault.dismiss();
    nav.replace('/start');
  }
</script>

<main class="gate">
  <div class="box">
    <div class="mark">tend<i>.</i></div>
    {#if vault.problem}
      <h3 class="h3">Can’t open your vault</h3>
      {#if vault.problem.path}<span class="mono">{vault.problem.path}</span>{/if}
      <div class="err" role="alert">{vault.problem.message}</div>
      {#if permission}
        <span class="cap">Allow Tend in System Settings → Privacy &amp; Security → Files and Folders, then try again.</span>
      {/if}
      <div class="acts">
        <button class="pbtn" onclick={vault.restore}>Try again</button>
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
    font: 600 32px/40px var(--font-heading);
    letter-spacing: -0.01em;
    margin-bottom: 8px;
  }

  .mark i {
    font-style: normal;
    color: var(--accent);
  }

  .h3 {
    font: 500 15px/20px var(--font-heading);
    margin: 0;
  }

  .mono {
    font: 400 13px/18px var(--font-mono);
    color: var(--text-muted);
    overflow-wrap: anywhere;
  }

  .cap {
    font: 400 13px/18px var(--font-ui);
    color: var(--c-muted-2);
  }

  .err {
    padding: 10px 12px;
    border-radius: 6px;
    background: var(--danger-soft);
    color: var(--on-danger-soft);
    font: 400 13px/18px var(--font-ui);
  }

  .acts {
    display: flex;
    gap: 8px;
    margin-top: 4px;
  }

  .pbtn {
    height: 30px;
    padding: 0 14px;
    border: 0;
    border-radius: 6px;
    background: var(--primary);
    color: var(--on-primary);
    font: 700 13px/18px var(--font-ui);
    cursor: pointer;
  }

  .pbtn:hover {
    background: var(--primary-hover);
  }

  .wait {
    display: flex;
    align-items: center;
    gap: 10px;
    font: 400 14px/20px var(--font-ui);
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
