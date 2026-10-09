<script lang="ts">
  // The color popover for a status or label: every swatch (the current one outlined), then any
  // color: the system color picker, or a typed #rrggbb (Enter applies it).
  import Popover from './Popover.svelte';
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { isHexColor, SWATCHES } from '#lib/util/swatches.ts';
  import { untrack } from 'svelte';

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

  // The custom color starts from the current one.
  let hex = $state(untrack(() => color?.toLowerCase() ?? ''));
  const valid = $derived(isHexColor(hex.trim()));

  function applyHex() {
    const value = hex.trim().toLowerCase();
    if (isHexColor(value)) onpick(value);
  }

  function hexKeys(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      applyHex();
    }
  }

  /** The system picker: follow it while it moves, apply when it closes. */
  function onSystemInput(e: Event) {
    hex = (e.currentTarget as HTMLInputElement).value;
  }

  function onSystemChange(e: Event) {
    hex = (e.currentTarget as HTMLInputElement).value;
    applyHex();
  }
</script>

<Popover {anchor} role="dialog" {label} {onclose}>
  <div class="swpop" role="listbox" aria-label={label}>
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
  <div class="custom">
    <input class="sys" type="color" aria-label="Pick any color" title="Pick any color" value={valid ? hex.trim() : (current ?? '#000000')} oninput={onSystemInput} onchange={onSystemChange} />
    <input use:plainText class="hex" class:bad={hex.trim() !== '' && !valid} aria-label="Hex color" placeholder="#rrggbb" maxlength="7" bind:value={hex} onkeydown={hexKeys} onfocus={(e) => e.currentTarget.select()} />
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

  .custom {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 10px 10px;
  }

  /* The native color well, drawn as a round swatch. */
  .sys {
    flex: 0 0 18px;
    width: 18px;
    height: 18px;
    padding: 0;
    border: 0;
    border-radius: 9px;
    background: none;
    cursor: pointer;
  }

  .sys::-webkit-color-swatch-wrapper {
    padding: 0;
  }

  .sys::-webkit-color-swatch {
    border: 1px solid var(--border);
    border-radius: 9px;
  }

  .hex {
    width: 0;
    flex: 1;
    height: 24px;
    padding: 0 6px;
    border: 0;
    border-radius: var(--radius-sm);
    outline: none;
    background: var(--c-hover);
    font: var(--text-mono-meta);
    color: var(--text);
  }

  .hex.bad {
    color: var(--danger);
  }

  .swopt.on {
    outline: 2px solid var(--text);
    outline-offset: 2px;
  }
</style>
