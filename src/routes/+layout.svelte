<script lang="ts">
  import { afterNavigate } from '$app/navigation';
  import '#lib/styles/app.css';
  import type { Snippet } from 'svelte';
  import { page } from '$app/state';
  import VaultGate from '#lib/components/VaultGate.svelte';
  import BottomNav from '#lib/components/BottomNav.svelte';
  import TopBar from '#lib/components/TopBar.svelte';
  import Toast from '#lib/components/Toast.svelte';
  import CommandPalette from '#lib/components/CommandPalette.svelte';
  import QuickCreate from '#lib/components/QuickCreate.svelte';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { timers } from '#lib/stores/timers.svelte.ts';
  import { available, blocked, commands } from '#lib/commands.ts';
  import { matches } from '#lib/util/keys.ts';
  import { routes } from '#lib/util/routes.ts';
  import { inTauri, watchFilled } from '#lib/util/window.ts';
  import { vault } from '#lib/stores/vault.svelte.ts';
  import { nav } from '#lib/stores/history.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { installErrorHandler } from '#lib/errors/handler.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { settingDefaults, settingKeys } from '#lib/config/settings.config.ts';
  import { applyTheme } from '#lib/util/theme.ts';
  import { startTabbingIn } from '#lib/util/focus-start.ts';
  import type { ThemePref } from '#lib/types/theme.type.ts';

  let { children }: { children: Snippet } = $props();

  // In the built app the first URL is "tauri://localhost", whose pathname is "" rather than "/".
  const path = $derived(page.url.pathname || routes.home);
  const onStart = $derived(path === routes.start);
  const settled = $derived(vault.ready && !vault.problem);
  /** A vault is open and the start page is behind us: the bottom bar and shortcuts are live. */
  const inVault = $derived(Boolean(vault.path) && !onStart);

  let pageEl = $state<HTMLElement>();

  // After a navigation nothing has focus; Tab should start in the page, not at the window buttons.
  afterNavigate((to) => {
    nav.record(to);
    const nothingFocused = document.activeElement === document.body;
    if (nothingFocused && pageEl) startTabbingIn(pageEl);
  });

  // Errors nobody handled (a failed command, a thrown error) show here as a toast.
  $effect(() => installErrorHandler(toast.error));

  // Once launch is settled: no vault → start page; "/" → Notes.
  $effect(() => {
    if (!settled) return;
    if (!vault.path && !onStart) nav.replace(routes.start);
    else if (vault.path && path === routes.home) nav.replace(routes.notes);
  });

  // Light or dark, from Settings (System until a vault's settings say otherwise).
  $effect(() => applyTheme(settings.get<ThemePref>(settingKeys.theme, settingDefaults.theme)));

  // Timers run as soon as a vault is open, whichever page is showing.
  $effect(() => {
    if (!vault.path) return;
    timers.start();
    return timers.stop;
  });

  // Square corners when the window fills the screen.
  let filled = $state(false);
  $effect(() => {
    if (!inTauri()) return;
    return watchFilled((value) => (filled = value));
  });

  function commandFor(e: KeyboardEvent) {
    if (!inVault) return undefined;
    return commands.find((c) => c.keys && matches(e, c.keys) && available(c));
  }

  function onkeydown(e: KeyboardEvent) {
    const cmd = commandFor(e);
    if (cmd) {
      e.preventDefault();
      cmd.run();
      return;
    }
    const isBlocked = blocked.some((k) => matches(e, k));
    if (isBlocked) e.preventDefault();
  }
</script>

<svelte:window {onkeydown} />

<div class="shell" class:filled>
  <TopBar />
  {#if !settled}
    <VaultGate />
  {:else}
    <div class="page" bind:this={pageEl}>{@render children()}</div>
    {#if inVault}<BottomNav />{/if}
  {/if}
</div>
{#if settled}
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
