<script lang="ts">
  // A centered dialog over the window. Esc or a click on the backdrop closes it.
  import type { Snippet } from 'svelte';
  import { pushLayer } from '#lib/util/layers.ts';

  let {
    onclose,
    width = 520,
    top = 72,
    shade = false,
    label,
    role = 'dialog',
    children,
  }: {
    onclose: () => void;
    width?: number;
    top?: number;
    /** Tint the window behind the dialog (confirmations). */
    shade?: boolean;
    label: string;
    role?: string;
    children: Snippet;
  } = $props();

  $effect(() => pushLayer(() => onclose()));
</script>

<div class="back" class:shade onpointerdown={onclose} aria-hidden="true"></div>
<div class="box" {role} aria-modal="true" aria-label={label} style:width="{width}px" style:top="{top}px">
  {@render children()}
</div>

<style>
  .back {
    position: fixed;
    inset: 0;
    z-index: 40;
  }

  .shade {
    background: rgba(48, 56, 65, 0.08);
  }

  .box {
    position: fixed;
    left: 50%;
    z-index: 41;
    max-width: calc(100vw - 32px);
    transform: translateX(-50%);
    background: var(--c-page);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text);
  }
</style>
