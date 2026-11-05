<script lang="ts">
  // A question as a sheet from the bottom of the phone (MD-Delete): title, a line of text, the
  // action as a wide button, Cancel under it. Tapping the veil or Esc cancels. `confirmWord` makes
  // the user type a word first (deleting a folder). The phone's form of ConfirmDialog.
  import { onMount } from 'svelte';
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { pushLayer } from '#lib/util/layers.ts';

  let {
    title,
    body,
    confirm,
    cancel = 'Cancel',
    danger = false,
    confirmWord,
    onconfirm,
    onclose,
  }: {
    title: string;
    body: string;
    confirm: string;
    cancel?: string;
    danger?: boolean;
    confirmWord?: string;
    onconfirm: () => void | Promise<void>;
    onclose: () => void;
  } = $props();

  let typed = $state('');
  let busy = $state(false);
  const blocked = $derived(confirmWord !== undefined && typed !== confirmWord);

  onMount(() => pushLayer(onclose));

  async function run() {
    if (blocked || busy) return;
    busy = true;
    try {
      await onconfirm();
    } finally {
      busy = false;
    }
  }
</script>

<button class="veil" tabindex="-1" aria-hidden="true" onclick={onclose}></button>
<div class="sheet" role="alertdialog" aria-modal="true" aria-label={title}>
  <h2>{title}</h2>
  <p>{body}</p>
  {#if confirmWord !== undefined}
    <p>Type <span class="key">{confirmWord}</span> to confirm.</p>
    <input use:plainText class="field" aria-label="Name to confirm" autocomplete="off" spellcheck="false" bind:value={typed} />
  {/if}
  <button class="act" class:danger disabled={blocked || busy} onclick={run}>{confirm}</button>
  <button class="cancel" onclick={onclose}>{cancel}</button>
</div>

<style>
  .veil {
    position: fixed;
    inset: 0;
    z-index: 60;
    padding: 0;
    border: 0;
    background: rgba(37, 44, 51, 0.08);
  }

  .sheet {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 61;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 28px 20px calc(16px + env(safe-area-inset-bottom));
    border-radius: 20px 20px 0 0;
    background: var(--c-page);
    box-shadow: 0 0 0 1px var(--border);
    text-align: center;
  }

  h2 {
    margin: 0;
    font: var(--text-form-main);
  }

  p {
    margin: 0;
    font: var(--text-ui);
    color: var(--text-muted);
  }

  .key {
    font: var(--text-mono-strong);
    color: var(--text);
  }

  .field {
    height: 44px;
    padding: 0 14px;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    background: var(--c-chip);
    font: var(--text-mono);
    color: var(--text);
  }

  .field:focus {
    outline: none;
    border-color: var(--danger);
  }

  .act,
  .cancel {
    height: 48px;
    border: 0;
    border-radius: var(--radius-md);
    font: var(--text-ui);
    font-size: 16px;
  }

  .act {
    margin-top: 8px;
    background: var(--primary);
    color: var(--on-primary);
  }

  .act.danger {
    background: var(--danger-soft);
    color: var(--danger);
  }

  .act:disabled {
    opacity: 0.5;
  }

  .cancel {
    background: transparent;
    color: var(--text-muted);
  }

  .cancel:focus-visible,
  .act:focus-visible {
    outline: none;
    filter: brightness(0.96);
  }
</style>
