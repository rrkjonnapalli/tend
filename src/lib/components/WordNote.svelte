<script lang="ts">
  // One of a word's markdown fields (Meaning, Notes), shown formatted. The pencil or a double-click
  // opens it for writing, empty or not, like every markdown field. ⌘↵ or ✓ saves, Esc or ✕ cancels.
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
</script>

<div class="note">
  <div class="head">
    <span class="field-label">{label}</span>
    <EditControls {editing} what={label.toLowerCase()} onedit={() => field?.edit()} oncancel={() => field?.cancel()} onsave={() => field?.save()} />
  </div>
  <MarkdownField bind:this={field} bind:editing {value} {onsave} {placeholder} empty={placeholder} minHeight={72} {label} />
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
</style>
