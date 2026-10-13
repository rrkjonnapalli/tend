<script lang="ts">
  // One of a word's markdown fields (Meaning, Notes), shown formatted. Empty, its prompt is a
  // button that opens it for writing; with text, the pencil or a double-click does. ⌘↵ or ✓ saves,
  // Esc or ✕ cancels.
  import MarkdownField from './MarkdownField.svelte';
  import EditControls from './EditControls.svelte';

  let {
    label,
    value,
    placeholder,
    onsave,
  }: { label: string; value: string; placeholder: string; onsave: (text: string) => void | Promise<void> } = $props();

  let field = $state<MarkdownField>();
  let editing = $state(false);

  const blank = $derived(!editing && value.trim() === '');
</script>

<div class="note">
  <div class="head">
    <span class="field-label">{label}</span>
    <EditControls {editing} what={label.toLowerCase()} onedit={() => field?.edit()} oncancel={() => field?.cancel()} onsave={() => field?.save()} />
  </div>
  {#if blank}
    <button class="prompt" onclick={() => field?.edit()}>{placeholder}</button>
  {/if}
  <div hidden={blank}>
    <MarkdownField bind:this={field} bind:editing {value} {onsave} {placeholder} minHeight={72} {label} />
  </div>
</div>

<style>
  .note {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .head {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 24px;
  }

  .prompt {
    align-self: flex-start;
    padding: 6px 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    font: var(--text-editor-sm);
    color: var(--c-muted-2);
    cursor: text;
  }
</style>
