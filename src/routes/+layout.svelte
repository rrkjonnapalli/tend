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
  import SnapDialogs from '#lib/components/SnapDialogs.svelte';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { timers } from '#lib/stores/timers.svelte.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { appAPI } from '#lib/api/app.ts';
  import { runBeforeQuit } from '#lib/util/before-quit.ts';
  import { available, blocked, commands } from '#lib/commands.ts';
  import { routes } from '#lib/util/routes.ts';
  import { inTauri, watchFilled } from '#lib/util/window.ts';
  import { vault } from '#lib/stores/vault.svelte.ts';
  import { update } from '#lib/stores/update.svelte.ts';
  import { nav } from '#lib/stores/history.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { installErrorHandler } from '#lib/errors/handler.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { settingDefaults, settingKeys } from '#lib/config/settings.config.ts';
  import { applyTheme } from '#lib/util/theme.ts';
  import { startTabbingIn } from '#lib/util/focus-start.ts';
  import { focusDock, focusMain, focusSide } from '#lib/util/panes.ts';
  import { hasLayers } from '#lib/util/layers.ts';
  import { isTypingTarget, matches } from '#lib/util/keys.ts';
  import { tick } from 'svelte';
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

  // On quit, pages save what they hold first (an open note), then Rust may close the app.
  $effect(() => {
    if (!inTauri()) return;
    const stop = appAPI.onQuitting(async () => {
      await runBeforeQuit();
      await appAPI.flushed();
    });
    return () => void stop.then((unlisten) => unlisten());
  });

  // Once per launch: look for a newer Tend.
  $effect(() => {
    if (inTauri()) void update.start();
  });

  // Back in the window after midnight: My day, Overdue and Upcoming move to the new day.
  $effect(() => {
    if (!vault.path) return;
    window.addEventListener('focus', tasks.ensureToday);
    return () => window.removeEventListener('focus', tasks.ensureToday);
  });

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
    // In a field, arrow chords (⌘⇧↓ selects to the end) belong to the field.
    const typingArrow = isTypingTarget(e.target) && e.key.startsWith('Arrow');
    if (!inVault || typingArrow) return undefined;
    return commands.find((c) => c.keys && matches(e, c.keys) && available(c));
  }

  /** The last thing focused in the page's main content, for ⌘→ to return to. */
  let lastInMain: HTMLElement | null = null;

  function onfocusin(e: FocusEvent) {
    const main = pageEl?.querySelector('main');
    const target = e.target as HTMLElement;
    if (main?.contains(target)) lastInMain = target;
  }

  /** ⌘← or ⌘→ outside a field (in a field they move the caret to the line's start or end). */
  function isPaneChord(e: KeyboardEvent): boolean {
    const isPaneKey = e.key.startsWith('Arrow');
    const onlyCmd = e.metaKey && !e.shiftKey && !e.altKey && !e.ctrlKey;
    return isPaneKey && onlyCmd && !isTypingTarget(e.target);
  }

  /** ⌘← to the explorer (opening it if hidden) or the Settings sections, ⌘↓ to the dock (opening
   *  it if hidden), ⌘→ / ⌘↑ back to the page. */
  async function paneKey(e: KeyboardEvent): Promise<boolean> {
    if (!inVault || !pageEl) return false;
    e.preventDefault();
    if (e.key === 'ArrowRight' || e.key === 'ArrowUp') return focusMain(pageEl, lastInMain);
    if (e.key === 'ArrowDown') {
      if (!ui.navOpen) ui.toggleNav();
      await tick();
      return focusDock(document);
    }
    if (!focusSide(pageEl) && !ui.explorerOpen) {
      ui.toggleExplorer();
      await tick();
      return focusSide(pageEl);
    }
    return true;
  }

  /** An arrow key while nothing in the page has focus (after a page change) enters the page. */
  function enterPage(e: KeyboardEvent): boolean {
    const isArrow = e.key.startsWith('Arrow');
    const hasModifier = e.metaKey || e.ctrlKey || e.altKey || e.shiftKey;
    const target = e.target as HTMLElement;
    const nothingFocused = target === document.body || target === pageEl;
    if (!inVault || !pageEl || !isArrow || hasModifier || !nothingFocused) return false;
    e.preventDefault();
    return focusMain(pageEl, lastInMain);
  }

  /** ↑ or Esc on the dock returns to the page. */
  function leaveDock(e: KeyboardEvent): boolean {
    const inDock = (e.target as HTMLElement).closest('[data-pane="dock"]') !== null;
    const isLeaveKey = e.key === 'ArrowUp' || e.key === 'Escape';
    if (!inDock || !isLeaveKey || hasLayers() || !pageEl) return false;
    e.preventDefault();
    return focusMain(pageEl, lastInMain);
  }

  /** Space presses a focused link or button-like element, as it does a real button. */
  function spacePresses(e: KeyboardEvent): boolean {
    const target = e.target as HTMLElement;
    const pressable = target.matches('a[href], [role="button"]');
    const plain = !e.metaKey && !e.ctrlKey && !e.altKey && !e.shiftKey;
    if (e.key !== ' ' || !plain || !pressable || e.defaultPrevented) return false;
    e.preventDefault();
    target.click();
    return true;
  }

  function onkeydown(e: KeyboardEvent) {
    // An open dropdown or dialog owns plain keys (arrows, Space) until it closes.
    const ownedByLayer = hasLayers() && !e.metaKey && !e.ctrlKey;
    if (!ownedByLayer && (enterPage(e) || leaveDock(e) || spacePresses(e))) return;
    if (isPaneChord(e)) {
      paneKey(e);
      return;
    }
    // The editor already used this key (⌘[ outdents a line): it isn't also an app shortcut.
    if (e.defaultPrevented) return;
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

<svelte:window {onkeydown} {onfocusin} />

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
  {#if inVault}<SnapDialogs />{/if}
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
