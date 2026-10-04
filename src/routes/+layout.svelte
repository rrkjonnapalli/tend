<script lang="ts">
  import '#lib/styles/app.css';
  import type { Snippet } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import VaultGate from '#lib/components/VaultGate.svelte';
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

  // In the built app the first URL is "tauri://localhost", whose pathname is "" rather than "/".
  const path = $derived(page.url.pathname || '/');
  const onStart = $derived(path === '/start');

  // Once launch is settled: no vault → start page; "/" → Notes.
  $effect(() => {
    if (!vault.ready || vault.problem) return;
    if (!vault.path && !onStart) goto('/start', { replace: true });
    else if (vault.path && path === '/') goto('/notes', { replace: true });
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
  {#if !vault.ready || vault.problem}
    <VaultGate />
  {:else}
    <div class="page">{@render children()}</div>
    {#if !onStart && vault.path}<BottomNav />{/if}
  {/if}
</div>
{#if vault.ready && !vault.problem}
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
