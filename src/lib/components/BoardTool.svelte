<script lang="ts">
  // One small icon button in a board item's corner (pin, rename, delete).
  import Icon from './Icon.svelte';
  import type { IconName } from '#lib/types/icon.type.ts';

  let {
    icon,
    label,
    title,
    pressed,
    danger = false,
    onclick,
  }: {
    icon: IconName;
    label: string;
    title: string;
    /** For a toggle: whether it is on. */
    pressed?: boolean;
    danger?: boolean;
    onclick: () => void;
  } = $props();
</script>

<button class="tool" class:on={pressed} class:danger aria-label={label} {title} aria-pressed={pressed} {onclick}>
  <Icon name={icon} size={13} />
</button>

<style>
  .tool {
    width: 22px;
    height: 22px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--board-ink-muted);
    cursor: pointer;
  }

  .tool:hover,
  .tool:focus-visible {
    background: var(--board-hover);
    color: var(--board-ink);
  }

  .tool.on {
    color: var(--pin-on);
  }

  .danger:hover,
  .danger:focus-visible {
    color: var(--danger);
  }
</style>
