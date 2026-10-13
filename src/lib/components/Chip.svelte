<script lang="ts">
  // A small chip (tags, labels, a word's synonyms): optional color dot · text (cut off with …) ·
  // red ✕ when it can be removed.
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
    removeLabel?: string;
    /** Leave it out for a chip that is only read: no ✕. */
    onremove?: () => void;
  } = $props();
</script>

<span class="chip" class:fixed={!onremove} title={text}>
  {#if color}<span class="dot" style:background={color}></span>{/if}
  <span class="tl">{text}</span>
  {#if onremove}
    <button class="x" aria-label={removeLabel} title="Remove" onclick={onremove}>
      <Icon name="close" size={9} stroke={3} />
    </button>
  {/if}
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

  /* Without the ✕ the text sits in the middle. */
  .fixed {
    padding-right: 8px;
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
