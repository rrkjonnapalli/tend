<script lang="ts">
  // "notes / work / tauri" with every part but the last clickable.
  import type { Crumb } from '#lib/types/breadcrumb.type.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';

  let { crumbs }: { crumbs: Crumb[] } = $props();

  // A phone has room for one line: the parent and the page (MD-Note).
  const shown = $derived(ui.phone ? crumbs.slice(-2) : crumbs);
</script>

<nav class="crumbs caption" class:one={ui.phone} aria-label="Breadcrumb">
  {#each shown as c, i (i)}
    {@const isLast = i === shown.length - 1}
    {#if i > 0}<span aria-hidden="true">/</span>{/if}
    {#if c.href && !isLast}
      <a class="crumb" href={c.href}>{c.label}</a>
    {:else}
      <span class="cur" aria-current={isLast ? 'page' : undefined}>{c.label}</span>
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

  .crumbs.one {
    flex-wrap: nowrap;
    overflow: hidden;
  }

  .one .cur {
    min-width: 0;
    font-weight: 700;
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
