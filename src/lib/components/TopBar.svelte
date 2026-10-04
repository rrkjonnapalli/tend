<script lang="ts">
  // The window's own title bar (native decorations are off): drag anywhere on it, double-click
  // to maximise. macOS gets window dots on the left; Windows and Linux get buttons on the right.
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { page } from '$app/state';
  import Icon from './Icon.svelte';
  import { isMac } from '#lib/util/keys.ts';

  const SECTIONS: Record<string, string> = { notes: 'Notes', tasks: 'Tasks', people: 'People', settings: 'Settings' };
  const title = $derived(SECTIONS[page.url.pathname.split('/')[1]] ?? 'Tend');

  const inTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  const win = () => getCurrentWindow();
</script>

<header class="bar" class:mac={isMac} data-tauri-drag-region>
  {#if isMac}
    <div class="dots" role="group" aria-label="Window">
      <button class="dot close" aria-label="Close window" onclick={() => inTauri && win().close()}><Icon name="close" size={8} stroke={3} /></button>
      <button class="dot min" aria-label="Minimize window" onclick={() => inTauri && win().minimize()}><Icon name="minimize" size={8} stroke={3} /></button>
      <button class="dot max" aria-label="Maximize window" onclick={() => inTauri && win().toggleMaximize()}><Icon name="plus" size={8} stroke={3} /></button>
    </div>
  {/if}
  <span class="title" data-tauri-drag-region>{title}</span>
  {#if !isMac}
    <div class="ctrls" role="group" aria-label="Window">
      <button class="ctrl" aria-label="Minimize window" title="Minimize" onclick={() => inTauri && win().minimize()}><Icon name="minimize" size={12} stroke={1.5} /></button>
      <button class="ctrl" aria-label="Maximize window" title="Maximize" onclick={() => inTauri && win().toggleMaximize()}><Icon name="maximize" size={11} stroke={1.5} /></button>
      <button class="ctrl close" aria-label="Close window" title="Close" onclick={() => inTauri && win().close()}><Icon name="close" size={12} stroke={1.5} /></button>
    </div>
  {/if}
</header>

<style>
  .bar {
    position: relative;
    flex: 0 0 32px;
    display: flex;
    align-items: center;
    border-bottom: 1px solid var(--border);
    background: var(--c-page);
    user-select: none;
  }

  .title {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    font: 400 12px/16px var(--font-ui);
    color: var(--c-muted-2);
    pointer-events: none;
  }

  /* macOS: quiet grey dots that take their colour while the group is hovered. */
  .dots {
    display: flex;
    gap: 8px;
    padding-left: 12px;
  }

  .dot {
    width: 12px;
    height: 12px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: var(--border);
    color: transparent;
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .dots:hover .close {
    background: var(--red-400);
  }

  .dots:hover .min {
    background: var(--orange-300);
  }

  .dots:hover .max {
    background: var(--green-400);
  }

  .dots:hover .dot {
    color: var(--text-muted);
  }

  /* Windows and Linux. */
  .ctrls {
    display: flex;
    margin-left: auto;
    align-self: stretch;
  }

  .ctrl {
    width: 44px;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--text-muted);
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .ctrl:hover {
    background: var(--c-hover);
    color: var(--text);
  }

  .ctrl.close:hover {
    background: var(--danger);
    color: var(--on-danger);
  }
</style>
