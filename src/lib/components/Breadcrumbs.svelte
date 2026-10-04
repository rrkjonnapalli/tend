<script lang="ts">
  // "notes / work / tauri" with every part but the last clickable.
  import type { Crumb } from '#lib/types/breadcrumb.type.ts';
  let { crumbs }: { crumbs: Crumb[] } = $props();
</script>

<nav class="crumbs caption" aria-label="Breadcrumb">
  {#each crumbs as c, i (i)}
    {#if i > 0}<span aria-hidden="true">/</span>{/if}
    {#if c.href && i < crumbs.length - 1}
      <a class="crumb" href={c.href}>{c.label}</a>
    {:else}
      <span class="cur" aria-current={i === crumbs.length - 1 ? 'page' : undefined}>{c.label}</span>
    {/if}
  {/each}
</nav>

<style>
  .crumbs {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
    min-width: 0;
    min-height: 24px;
    color: var(--c-muted-2);
  }

  .crumb {
    color: var(--text-muted);
    text-decoration: none;
  }

  .crumb:hover {
    color: var(--primary);
  }

  .cur {
    max-width: 360px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text);
  }
</style>
