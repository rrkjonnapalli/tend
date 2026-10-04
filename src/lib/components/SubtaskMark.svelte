<script lang="ts">
  // A tiny "s" badge before a subtask's title in task lists and on board cards; hover says "Subtask".
  import type { Task } from '#lib/types/task.type.ts';

  let { task }: { task: Task } = $props();

  const HINT = 'Subtask';
</script>

{#if task.parentId}
  <span class="mark" title={HINT} aria-label={HINT} role="img">s</span>
{/if}

<style>
  .mark {
    display: inline-grid;
    place-items: center;
    width: 14px;
    height: 14px;
    margin-right: 6px;
    border-radius: 50%;
    background: var(--success-soft);
    color: var(--success);
    font: 700 9px/1 var(--font-ui);
    vertical-align: 1px;
    cursor: default;
  }
</style>
