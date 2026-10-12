<script lang="ts" generics="T extends string">
  // A segmented switch: one choice of a few. Radios underneath, so ←→ moves the choice and screen
  // readers hear a radio group; the selected segment is a page-coloured pill.
  let {
    options,
    value,
    label,
    onchange,
  }: {
    options: { value: T; label: string }[];
    value: T;
    /** Accessible name of the group. */
    label: string;
    onchange: (value: T) => void;
  } = $props();

  const name = $props.id();
</script>

<div class="seg" role="radiogroup" aria-label={label}>
  {#each options as o (o.value)}
    <label class="choice" class:on={value === o.value}>
      <input type="radio" {name} value={o.value} checked={value === o.value} onchange={() => onchange(o.value)} />
      {o.label}
    </label>
  {/each}
</div>

<style>
  .seg {
    display: inline-flex;
    flex: none;
    gap: 2px;
    padding: 3px;
    border-radius: var(--radius-control);
    background: var(--c-chip);
  }

  .choice {
    position: relative;
    display: inline-flex;
    align-items: center;
    height: 28px;
    padding: 0 14px;
    border-radius: var(--radius-sm);
    font: var(--text-ui);
    color: var(--c-muted-2);
    cursor: pointer;
  }

  .choice:hover {
    color: var(--text);
  }

  .choice.on {
    background: var(--c-page);
    color: var(--text);
  }

  .choice:has(input:focus-visible) {
    background: var(--c-hover);
    color: var(--text);
  }

  .choice input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }
</style>
