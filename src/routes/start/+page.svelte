<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { goto } from '$app/navigation';
  import Icon from '#lib/components/Icon.svelte';
  import { vaultApi, type Check } from '#lib/api/vault.ts';
  import { messageOf } from '#lib/api/core.ts';
  import { vault } from '#lib/stores/vault.svelte.ts';
  import { isMac } from '#lib/util/keys.ts';

  const mod = isMac ? '⌘' : 'Ctrl';
  const STEP_MS = 450;

  let check = $state<Check | null>(null);
  let shown = $state(0);
  let failure = $state<string | null>(null);
  let timer: ReturnType<typeof setInterval> | undefined;

  const done = $derived(check !== null && shown >= check.steps.length);

  async function choose() {
    const picked = await open({ directory: true, title: 'Choose vault folder' });
    if (typeof picked !== 'string') return;
    clearInterval(timer);
    failure = null;
    shown = 0;
    try {
      check = await vaultApi.open(picked);
    } catch (e) {
      check = null;
      failure = messageOf(e);
      return;
    }
    timer = setInterval(() => {
      shown += 1;
      if (check && shown >= check.steps.length) clearInterval(timer);
    }, STEP_MS);
  }

  function reset() {
    clearInterval(timer);
    check = null;
    shown = 0;
  }

  async function enter() {
    if (!check) return;
    await vault.opened(check.path);
    goto('/notes', { replace: true });
  }

  const examples = [
    ['task: Write README', 'adds a task'],
    ['person: Ravi Kumar', 'adds a person'],
    ['timer 10m tea', 'timer'],
  ];
</script>

<main class="start">
  <div class="cols">
    <div class="col">
      <div class="brand">
        <div class="mark">tend<i>.</i></div>
        <span class="cap">Notes · Tasks · People</span>
      </div>
      <section class="sec">
        <h3 class="h3">Start</h3>
        {#if !check}
          <button class="act" onclick={choose}><Icon name="folder" size={15} />Choose vault folder…</button>
          <div class="vlist">
            <div><span class="mono">notes/</span><span class="cap">your notes, used if it exists</span></div>
            <div><span class="mono">.attachments/</span><span class="cap">attached files, created if missing</span></div>
            <div><span class="mono">db/app.sqlite</span><span class="cap">everything else, created if missing</span></div>
          </div>
          {#if failure}<div class="err" role="alert">{failure}</div>{/if}
        {:else}
          <div class="path">
            <span class="folder"><Icon name="folder" size={15} /></span>
            <span class="p">{check.path}</span>
            <button class="gbtn sm" onclick={reset}>Change</button>
          </div>
          <div class="steps">
            {#each check.steps as st, i (st.item)}
              {@const state = i < shown ? (st.status === 'error' ? 'bad' : 'ok') : i === shown ? 'run' : 'wait'}
              <div class="step {state}">
                <span class="ic">
                  {#if state === 'ok'}<span class="okic"><Icon name="check" size={13} stroke={2.5} /></span>{/if}
                  {#if state === 'bad'}<span class="badic"><Icon name="close" size={13} stroke={2.5} /></span>{/if}
                </span>
                <span class="mono name">{st.item}</span>
                <span class="st">{i < shown ? st.detail : state === 'run' ? 'checking…' : ''}</span>
              </div>
            {/each}
          </div>
          {#if done && !check.ok}
            <div class="err" role="alert">
              An incompatible database exists in <span class="mono">db/app.sqlite</span>. Choose another folder, or
              move that file out of the way and try again.
            </div>
            <div><button class="pbtn" onclick={choose}>Choose another folder</button></div>
          {/if}
          {#if done && check.ok}
            <div><button class="pbtn" onclick={enter}>Open vault →</button></div>
          {/if}
        {/if}
      </section>
    </div>
    <div class="col right">
      <section class="sec">
        <h3 class="h3">Command palette</h3>
        <span class="cap">Press <kbd>{mod}</kbd><kbd>K</kbd> anywhere. Type to search, or write what you want:</span>
        {#each examples as [text, what] (text)}
          <div class="cmd"><span class="mono">{text}</span>{what}</div>
        {/each}
      </section>
      <section class="sec">
        <h3 class="h3">Shortcuts</h3>
        <div class="cap keys">
          <span><kbd>{mod}</kbd><kbd>1</kbd>–<kbd>3</kbd></span><span>switch sections</span>
          <span><kbd>{mod}</kbd><kbd>N</kbd></span><span>new note</span>
          <span><kbd>{mod}</kbd><kbd>\</kbd></span><span>toggle explorer</span>
        </div>
      </section>
    </div>
  </div>
</main>

<style>
  .start {
    flex: 1;
    overflow: auto;
  }

  .cols {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 72px;
    padding: 112px 0 0;
    width: 960px;
    max-width: 100%;
    margin: 0 auto;
  }

  .col {
    display: flex;
    flex-direction: column;
    gap: 36px;
  }

  .col.right {
    padding-top: 82px;
  }

  .brand {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .mark {
    font: 600 32px/40px var(--font-heading);
    letter-spacing: -0.01em;
  }

  .mark i {
    font-style: normal;
    color: var(--accent);
  }

  .brand .cap {
    font-size: 14px;
  }

  .h3 {
    font: 500 15px/20px var(--font-heading);
    margin: 0;
  }

  .cap {
    font: 400 13px/18px var(--font-ui);
    color: var(--c-muted-2);
  }

  .mono {
    font: 400 12px/18px var(--font-mono);
    color: var(--text-muted);
  }

  .sec {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .act {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 34px;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--primary);
    font: 400 14px/20px var(--font-ui);
    cursor: pointer;
    align-self: flex-start;
  }

  .act:hover {
    color: var(--primary-hover);
    text-decoration: underline;
  }

  .vlist {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .vlist div {
    display: flex;
    align-items: baseline;
    gap: 12px;
  }

  .vlist .mono {
    flex: 0 0 110px;
  }

  .path {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 40px;
    padding: 0 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }

  .path .folder {
    display: grid;
    color: var(--c-muted-2);
  }

  .path .p {
    flex: 1;
    min-width: 0;
    font: 400 13px/18px var(--font-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .gbtn {
    display: inline-flex;
    align-items: center;
    height: 28px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--c-page);
    color: var(--text);
    font: 400 13px/18px var(--font-ui);
    cursor: pointer;
  }

  .gbtn:hover {
    border-color: var(--c-muted-2);
  }

  .steps {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-left: 4px;
  }

  .step {
    display: flex;
    align-items: center;
    gap: 10px;
    font: 400 14px/20px var(--font-ui);
  }

  .step .name {
    flex: 0 0 120px;
  }

  .step .ic {
    flex: 0 0 14px;
    display: grid;
    place-items: center;
  }

  .okic {
    display: grid;
    color: var(--success);
  }

  .badic {
    display: grid;
    color: var(--danger);
  }

  .step .st {
    color: var(--c-muted-2);
    font-size: 13px;
  }

  .step.wait .name {
    color: var(--c-faint);
  }

  .step.wait .ic::before {
    content: '';
    width: 6px;
    height: 6px;
    border-radius: 3px;
    background: var(--border);
  }

  .step.run .ic::before {
    content: '';
    width: 10px;
    height: 10px;
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

  .err {
    padding: 10px 12px;
    border-radius: 6px;
    background: var(--danger-soft);
    color: var(--on-danger-soft);
    font: 400 13px/18px var(--font-ui);
  }

  .err .mono {
    color: var(--on-danger-soft);
  }

  .pbtn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 36px;
    padding: 0 16px;
    border: 0;
    border-radius: var(--radius-md);
    background: var(--primary);
    color: var(--on-primary);
    font: 700 14px/20px var(--font-ui);
    cursor: pointer;
  }

  .pbtn:hover {
    background: var(--primary-hover);
  }

  .cmd {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 34px;
    padding: 0 12px;
    border: 1px solid var(--border);
    border-radius: 6px;
    font: 400 13px/18px var(--font-ui);
    color: var(--text-muted);
  }

  .cmd .mono {
    color: var(--text);
    flex: 1;
  }

  .keys {
    display: grid;
    grid-template-columns: 90px 1fr;
    row-gap: 8px;
    align-items: center;
  }

  kbd {
    display: inline-grid;
    place-items: center;
    min-width: 20px;
    height: 20px;
    padding: 0 5px;
    border: 1px solid var(--border);
    border-bottom-width: 2px;
    border-radius: var(--radius-sm);
    background: var(--c-page);
    font: 400 11px/14px var(--font-mono);
    color: var(--text-muted);
  }
</style>
