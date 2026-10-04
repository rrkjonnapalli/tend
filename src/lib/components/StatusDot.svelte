<script lang="ts">
  // The status colour dot; the first status (Not started) is drawn as a ring.
  import { FIRST_STATUS } from '#lib/util/statuses.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';

  let { value, size = 8 }: { value: string; size?: number } = $props();

  const color = $derived(tasks.status(value)?.color ?? 'var(--c-muted-2)');
</script>

<span
  class="dot"
  class:ring={value === FIRST_STATUS}
  style:--c={color}
  style:width="{size}px"
  style:height="{size}px"
  aria-hidden="true"></span>

<style>
  .dot {
    flex: 0 0 auto;
    display: inline-block;
    border-radius: 50%;
    background: var(--c);
  }

  .ring {
    background: transparent;
    border: 1.5px solid var(--c);
  }
</style>
