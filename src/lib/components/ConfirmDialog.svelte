<script lang="ts">
  // A yes/no question in a modal: Esc or the left button cancels, the right button confirms.
  import Modal from './Modal.svelte';

  let {
    title,
    body,
    confirm,
    cancel = 'Cancel',
    onconfirm,
    onclose,
  }: {
    title: string;
    body: string;
    confirm: string;
    cancel?: string;
    onconfirm: () => void;
    onclose: () => void;
  } = $props();

  const WIDTH = 440;
  const TOP = 120;
</script>

<Modal label={title} role="alertdialog" width={WIDTH} top={TOP} shade {onclose}>
  <div class="dlg">
    <h2>{title}</h2>
    <span>{body}</span>
    <div class="foot">
      <button class="btn-ghost" onclick={onclose}>{cancel}</button>
      <!-- svelte-ignore a11y_autofocus -->
      <button class="btn-primary" autofocus onclick={onconfirm}>{confirm}</button>
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

  .foot {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
