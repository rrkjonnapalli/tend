<script lang="ts">
  // A yes/no question in a modal. Esc or the left button cancels; the right button runs
  // `onconfirm` and stays disabled until it settles, so a double click can't run it twice.
  // `danger` styles it for deleting; `confirmWord` makes the user type a word first.
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import Modal from './Modal.svelte';

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
    /** When set, the user types this to enable the button. */
    confirmWord?: string;
    onconfirm: () => void | Promise<void>;
    onclose: () => void;
  } = $props();

  const WIDTH = 440;
  const TOP = 120;

  let typed = $state('');
  let busy = $state(false);
  const blocked = $derived(confirmWord !== undefined && typed !== confirmWord);

  async function run() {
    if (blocked || busy) return;
    busy = true;
    try {
      await onconfirm();
    } finally {
      busy = false;
    }
  }

  function onFieldKey(e: KeyboardEvent) {
    if (e.key === 'Enter') run();
  }
</script>

<Modal label={title} role="alertdialog" width={WIDTH} top={TOP} shade {onclose}>
  <div class="dlg">
    <h2>{title}</h2>
    <span>{body}</span>
    {#if confirmWord !== undefined}
      <span>Type <span class="key">{confirmWord}</span> to confirm.</span>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        use:plainText
        class="field mono"
        class:armed={!blocked}
        aria-label="Name to confirm"
        autocomplete="off"
        spellcheck="false"
        autofocus
        bind:value={typed}
        onkeydown={onFieldKey} />
    {/if}
    <div class="foot">
      <button class="btn-ghost" onclick={onclose}>{cancel}</button>
      <!-- svelte-ignore a11y_autofocus -->
      <button
        class={danger ? 'btn-danger' : 'btn-primary'}
        disabled={blocked || busy}
        autofocus={confirmWord === undefined}
        onclick={run}>{confirm}</button>
    </div>
  </div>
</Modal>

<style>
  .dlg {
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: 12px;
    font: var(--text-ui);
    border-radius: var(--radius-md);
  }

  h2 {
    margin: 0;
    font: var(--text-section);
  }

  .key {
    font: 500 13px/18px var(--font-mono);
    padding: 0 var(--space-1);
    border-radius: 3px;
    background: var(--c-chip);
  }

  /* The word is typed: the next Enter deletes. */
  .field.armed {
    border-color: var(--danger);
  }

  .foot {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
  }
</style>
