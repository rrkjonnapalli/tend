<script lang="ts">
  // The ⋯ / right-click menu on an explorer row. The root folder gets Rename only.
  import type { Anchor } from '#lib/types/popover.type.ts';
  import Popover from './Popover.svelte';
  import { label as keyLabel } from '#lib/util/keys.ts';

  let {
    anchor,
    isRoot,
    onrename,
    onmove,
    ondelete,
    onclose,
  }: {
    anchor: Anchor;
    isRoot: boolean;
    onrename: () => void;
    onmove: () => void;
    ondelete: () => void;
    onclose: () => void;
  } = $props();
</script>

<Popover {anchor} placement="bottom-end" role="menu" pad {onclose}>
  <div class="menu">
    <button class="opt" role="menuitem" onclick={onrename}>Rename<span class="hint">F2</span></button>
    {#if !isRoot}
      <button class="opt" role="menuitem" onclick={onmove}>
        Move to…<span class="hint">{keyLabel('mod+shift+m')}</span>
      </button>
      <div class="opt-sep"></div>
      <button class="opt danger" role="menuitem" onclick={ondelete}>Delete…<span class="hint">⌫</span></button>
    {/if}
  </div>
</Popover>

<style>
  .menu {
    min-width: 180px;
  }

  .menu .opt {
    height: 30px;
    font-size: 14px;
  }
</style>
