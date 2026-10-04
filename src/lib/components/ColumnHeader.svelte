<script lang="ts">
  // The column titles above a list's rows. `template` is the same grid the rows use, so each
  // title sits over its column.
  let {
    columns,
    template,
    gap = 'var(--col-gap)',
  }: {
    /** One title per grid column; '' leaves a column untitled. */
    columns: string[];
    /** A grid-template-columns value, e.g. "var(--grid-task)". */
    template: string;
    gap?: string;
  } = $props();
</script>

<div class="head" role="presentation" style:grid-template-columns={template} style:column-gap={gap}>
  {#each columns as label, i (i)}
    <span class="col">{label}</span>
  {/each}
</div>

<style>
  .head {
    flex: 0 0 auto;
    display: grid;
    padding: 0 0 6px;
    border-bottom: 1px solid var(--c-chip);
    font: 400 12px/16px var(--font-ui);
    color: var(--c-faint);
  }

  .col {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
