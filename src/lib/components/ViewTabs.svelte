<script lang="ts" generics="T extends string">
  // A line of text tabs with a dot between them: the one that is on is underlined. Used inside a
  // `bar` zone, where ← → move along it and pick the tab the focus lands on (`data-pick`).
  let {
    options,
    value,
    onchange,
  }: {
    options: { value: T; label: string; count?: number }[];
    /** The tab that is on; a value none of them has leaves all off. */
    value: string;
    onchange: (value: T) => void;
  } = $props();
</script>

{#each options as o, i (o.value)}
  {#if i > 0}<span class="sep" aria-hidden="true">·</span>{/if}
  <button class="tab" data-pick class:on={value === o.value} aria-pressed={value === o.value} onclick={() => onchange(o.value)}>
    {o.label}{#if o.count !== undefined}<span class="count">{o.count}</span>{/if}
  </button>
{/each}

<style>
  .tab {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 28px;
    padding: 2px 2px 0;
    border: 0;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--c-muted-2);
    font: var(--text-ui);
    cursor: pointer;
  }

  .tab:hover {
    color: var(--text);
  }

  .tab.on {
    color: var(--c-current-ink);
    border-bottom-color: var(--c-current);
  }

  .count {
    font: var(--text-meta);
    color: var(--c-muted-2);
  }

  .sep {
    color: var(--c-muted-2);
    line-height: 28px;
  }
</style>
