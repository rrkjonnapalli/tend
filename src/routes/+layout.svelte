<script lang="ts">
  import '#lib/styles/app.css';
  import type { Snippet } from 'svelte';
  import BottomNav from '#lib/components/BottomNav.svelte';
  import { blocked, commands } from '#lib/commands.ts';
  import { matches } from '#lib/util/keys.ts';

  let { children }: { children: Snippet } = $props();

  function onkeydown(e: KeyboardEvent) {
    const cmd = commands.find((c) => c.keys && matches(e, c.keys));
    if (cmd) {
      e.preventDefault();
      cmd.run();
      return;
    }
    if (blocked.some((k) => matches(e, k))) e.preventDefault();
  }
</script>

<svelte:window {onkeydown} />

<div class="shell">
  <div class="page">{@render children()}</div>
  <BottomNav />
</div>

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
