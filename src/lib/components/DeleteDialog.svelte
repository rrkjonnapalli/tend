<script lang="ts">
  // Delete confirmation. For a folder, the folder name must be typed to enable the button.
  import Modal from './Modal.svelte';

  let {
    title,
    body,
    confirmWord,
    button = 'Delete',
    ondelete,
    onclose,
  }: {
    title: string;
    body: string;
    /** When set, the user types this to confirm. */
    confirmWord?: string;
    button?: string;
    ondelete: () => void;
    onclose: () => void;
  } = $props();

  let typed = $state('');
  const blocked = $derived(confirmWord !== undefined && typed !== confirmWord);

  function confirm() {
    if (!blocked) ondelete();
  }
</script>

<Modal label={title} role="alertdialog" width={440} top={120} shade {onclose}>
  <div class="dlg">
    <h2>{title}</h2>
    <span>{body}</span>
    {#if confirmWord !== undefined}
      <span>Type <span class="key">{confirmWord}</span> to confirm.</span>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        class="fld"
        class:ok={!blocked}
        aria-label="Folder name"
        autocomplete="off"
        spellcheck="false"
        autofocus
        bind:value={typed}
        onkeydown={(e) => e.key === 'Enter' && confirm()} />
    {/if}
    <div class="foot">
      <button class="btn-ghost" onclick={onclose}>Cancel</button>
      <!-- svelte-ignore a11y_autofocus -->
      <button class="del" disabled={blocked} autofocus={confirmWord === undefined} onclick={confirm}>{button}</button>
    </div>
  </div>
</Modal>

<style>
  .dlg {
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    font: 400 14px/20px var(--font-ui);
    border-radius: 10px;
  }

  h2 {
    margin: 0;
    font: 500 16px/22px var(--font-heading);
  }

  .key {
    font: 500 13px/18px var(--font-mono);
    padding: 0 4px;
    border-radius: 3px;
    background: var(--c-chip);
  }

  .fld {
    height: 34px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: 6px;
    outline: none;
    font: 400 14px/20px var(--font-mono);
  }

  .fld:focus {
    border-color: var(--primary);
  }

  .fld.ok {
    border-color: var(--danger);
  }

  .foot {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .del {
    height: 30px;
    padding: 0 12px;
    border: 0;
    border-radius: 6px;
    background: var(--danger);
    color: var(--on-danger);
    font: 700 13px/18px var(--font-ui);
    cursor: pointer;
  }

  .del:hover {
    background: var(--red-700);
  }

  .del:disabled {
    background: color-mix(in srgb, var(--danger) 18%, var(--c-page));
    cursor: default;
  }
</style>
