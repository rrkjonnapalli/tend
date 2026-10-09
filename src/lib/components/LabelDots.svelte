<script lang="ts">
  // A task's labels on a board card: a dot per label in its color (names are too long for a
  // card), up to four, then "+N". Hover lists every name.
  import { tasks } from '#lib/stores/tasks.svelte.ts';

  let { taskId }: { taskId: string } = $props();

  const MAX_DOTS = 4;

  const labels = $derived(tasks.labelsOf(taskId));
  const shown = $derived(labels.slice(0, MAX_DOTS));
  const more = $derived(labels.length - shown.length);
  const names = $derived(labels.map((l) => l.name).join(', '));
</script>

{#if labels.length}
  <span class="dots" title={names} aria-label="Labels: {names}" role="img">
    {#each shown as l (l.id)}<span class="dot" style:background={l.color}></span>{/each}
    {#if more > 0}<span class="more">+{more}</span>{/if}
  </span>
{/if}

<style>
  .dots {
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .more {
    margin-left: 1px;
    font: var(--text-tiny);
    color: var(--c-muted-2);
  }
</style>
