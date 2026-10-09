<script lang="ts">
  // A small removable chip (tags, labels): optional color dot · text (cut off with …) · red ✕.
  import Icon from './Icon.svelte';

  let {
    text,
    color,
    removeLabel,
    onremove,
  }: {
    text: string;
    /** A label's color, shown as a dot before the text. */
    color?: string;
    /** Accessible name of the ✕, e.g. "Remove tag ci". */
    removeLabel: string;
    onremove: () => void;
  } = $props();
</script>

<span class="chip" title={text}>
  {#if color}<span class="dot" style:background={color}></span>{/if}
  <span class="tl">{text}</span>
  <button class="x" aria-label={removeLabel} title="Remove" onclick={onremove}>
    <Icon name="close" size={9} stroke={3} />
  </button>
</span>

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    max-width: 152px;
    padding: 2px 4px 2px 8px;
    border-radius: var(--radius-sm);
    background: var(--c-hover);
    color: var(--text-muted);
    font: var(--text-caption);
  }

  .dot {
    flex: 0 0 8px;
    width: 8px;
    height: 8px;
    margin-right: 2px;
    border-radius: 50%;
  }

  .tl {
    min-width: 0;
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .x {
    flex: 0 0 14px;
    width: 14px;
    height: 14px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--danger);
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .x:hover {
    background: var(--danger-soft);
  }
</style>
