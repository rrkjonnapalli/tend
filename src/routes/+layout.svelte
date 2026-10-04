<script lang="ts">
  import '#lib/styles/app.css';
  import type { Snippet } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import BottomNav from '#lib/components/BottomNav.svelte';
  import Toast from '#lib/components/Toast.svelte';
  import { blocked, commands } from '#lib/commands.ts';
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

  function onkeydown(e: KeyboardEvent) {
    const cmd = vault.path ? commands.find((c) => c.keys && matches(e, c.keys)) : undefined;
    if (cmd) {
      e.preventDefault();
      cmd.run();
      return;
    }
    if (blocked.some((k) => matches(e, k))) e.preventDefault();
  }
</script>

<svelte:window {onkeydown} />

{#if vault.ready}
  <div class="shell">
    <div class="page">{@render children()}</div>
    {#if !onStart && vault.path}<BottomNav />{/if}
  </div>
  <Toast />
{/if}

<style>
  .shell {
    position: relative;
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .page {
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
  }
</style>
