<script lang="ts">
  // Marks a subtask in task lists and on board cards; hover names its parent.
  import type { Task } from '#lib/types/task.type.ts';
  import Icon from './Icon.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';

  let { task }: { task: Task } = $props();

  const parent = $derived(task.parentId ? tasks.byId(task.parentId) : undefined);
  const hint = $derived(parent ? `Subtask of ${parent.title}` : 'Subtask');
</script>

{#if task.parentId}
  <span class="mark" title={hint} aria-label={hint} role="img"><Icon name="subtask" size={13} /></span>
{/if}

<style>
  .mark {
    display: inline-grid;
    place-items: center;
    flex: 0 0 auto;
    margin-right: 4px;
    color: var(--c-muted-2);
    vertical-align: -2px;
  }
</style>
