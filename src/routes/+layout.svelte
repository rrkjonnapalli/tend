<script lang="ts">
  import '#lib/styles/app.css';
  import type { Snippet } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import BottomNav from '#lib/components/BottomNav.svelte';
  import TopBar from '#lib/components/TopBar.svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import Toast from '#lib/components/Toast.svelte';
  import CommandPalette from '#lib/components/CommandPalette.svelte';
  import QuickCreate from '#lib/components/QuickCreate.svelte';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { timers } from '#lib/stores/timers.svelte.ts';
  import { available, blocked, commands } from '#lib/commands.ts';
  import { matches } from '#lib/util/keys.ts';
  import { vault } from '#lib/stores/vault.svelte.ts';

  let { children }: { children: Snippet } = $props();

  /** In a plain browser during development, answer commands from fixtures (see lib/dev). */
  async function devMock() {
    if (import.meta.env.DEV && !('__TAURI_INTERNALS__' in window)) await import('#lib/dev/mock-ipc.ts');
  }

  const onStart = $derived(page.url.pathname === '/start');

  $effect(() => {
    if (vault.ready) return;
    devMock().then(vault.restore).then((ok) => {
      if (!ok) goto('/start', { replace: true });
      else if (page.url.pathname === '/' || onStart) goto('/notes', { replace: true });
    });
  });

  // Timers run as soon as a vault is open, whichever page is showing.
  $effect(() => {
    if (vault.path) timers.start();
  });

  // Square corners when the window fills the screen.
  let filled = $state(false);
  $effect(() => {
    if (!('__TAURI_INTERNALS__' in window)) return;
    const win = getCurrentWindow();
    const check = async () => (filled = (await win.isMaximized()) || (await win.isFullscreen()));
    check();
    const stop = win.onResized(check);
    return () => {
      stop.then((f) => f());
    };
  });

  function onkeydown(e: KeyboardEvent) {
    const cmd = vault.path && !onStart ? commands.find((c) => c.keys && matches(e, c.keys) && available(c)) : undefined;
    if (cmd) {
      e.preventDefault();
      cmd.run();
      return;
    }
    if (blocked.some((k) => matches(e, k))) e.preventDefault();
  }
</script>

<svelte:window {onkeydown} />

<div class="shell" class:filled>
  <TopBar />
  {#if vault.ready}
    <div class="page">{@render children()}</div>
    {#if !onStart && vault.path}<BottomNav />{/if}
  {/if}
</div>
{#if vault.ready}
  {#if ui.palette}<CommandPalette onclose={() => (ui.palette = false)} />{/if}
  {#if ui.quick}{#key ui.quick}<QuickCreate kind={ui.quick} onclose={() => (ui.quick = null)} />{/key}{/if}
  <Toast />
{/if}

<style>
  .shell {
    position: relative;
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--c-page);
    border: 1px solid var(--border);
    border-radius: var(--window-radius);
  }

  .shell.filled {
    border: 0;
    border-radius: 0;
  }

  .page {
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
  }
</style>
