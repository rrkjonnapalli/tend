<script lang="ts">
  // The person page's "Assigned tasks": open tasks first, then done ones.
  import Section from './Section.svelte';
  import StatusDot from './StatusDot.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { isDone } from '#lib/util/task-views.ts';
  import { routes } from '#lib/util/routes.ts';

  let { person }: { person: string } = $props();

  const assigned = $derived(
    tasks.tasks.filter((t) => t.assigneeId === person).sort((a, b) => Number(isDone(a)) - Number(isDone(b))),
  );
</script>

<Section title="Assigned tasks">
  {#each assigned as t (t.id)}
    <a class="trow" href={routes.task(t.id)}>
      <StatusDot value={t.status} />
      <span class:done={isDone(t)}>{t.title}</span>
    </a>
  {/each}
  {#if assigned.length === 0}<span class="caption muted none">No tasks assigned</span>{/if}
</Section>

<style>
  .trow {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 32px;
    padding: 0 16px;
    margin: 0 -16px;
    font: 400 14px/20px var(--font-ui);
    color: var(--text);
    text-decoration: none;
  }

  .trow:hover {
    background: var(--c-hover);
    color: var(--text);
  }

  .done {
    color: var(--c-muted-2);
  }

  .none {
    padding: 6px 0;
  }
</style>
