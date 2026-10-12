<script lang="ts">
  // One of a word's markdown fields (Meaning, Notes): shown formatted; a click on the text (not a
  // link) opens it for writing. ⌘↵ or ✓ saves, Esc or ✕ cancels.
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

  function openOnClick(e: MouseEvent) {
    const onLink = (e.target as HTMLElement).closest('a, button');
    if (!editing && !onLink) field?.edit();
  }
</script>

<div class="note">
  <div class="head">
    <span class="lbl">{label}</span>
    <EditControls {editing} what={label.toLowerCase()} onedit={() => field?.edit()} oncancel={() => field?.cancel()} onsave={() => field?.save()} />
  </div>
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div onclick={openOnClick}>
    <MarkdownField bind:this={field} bind:editing {value} {onsave} {placeholder} empty={placeholder} minHeight={72} {label} />
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

  .lbl {
    font: 700 12px/16px var(--font-ui);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-muted);
  }
</style>
