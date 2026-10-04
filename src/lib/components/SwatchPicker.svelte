<script lang="ts">
  // The color popover for a status or label: every swatch, the current one outlined.
  import Popover from './Popover.svelte';
  import { SWATCHES } from '#lib/util/swatches.ts';

  let {
    anchor,
    color,
    onpick,
    onclose,
    label,
  }: {
    anchor: HTMLElement;
    color: string | undefined;
    onpick: (hex: string) => void;
    onclose: () => void;
    /** Accessible name, e.g. "Status color". */
    label: string;
  } = $props();

  const current = $derived(color?.toLowerCase());
</script>

<Popover {anchor} role="listbox" {label} {onclose}>
  <div class="swpop">
    {#each SWATCHES as [name, hex] (hex)}
      <button
        class="swopt"
        class:on={current === hex}
        style:background={hex}
        role="option"
        aria-selected={current === hex}
        aria-label={name}
        title={name}
        onclick={() => onpick(hex)}></button>
    {/each}
  </div>
</Popover>

<style>
  .swpop {
    display: grid;
    grid-template-columns: repeat(5, 18px);
    gap: 8px;
    padding: 10px;
  }

  .swopt {
    width: 18px;
    height: 18px;
    padding: 0;
    border: 0;
    border-radius: 9px;
    cursor: pointer;
  }

  .swopt:hover {
    outline: 2px solid var(--c-faint);
    outline-offset: 2px;
  }

  .swopt.on {
    outline: 2px solid var(--text);
    outline-offset: 2px;
  }
</style>
