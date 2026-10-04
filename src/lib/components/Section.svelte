<script lang="ts">
  // A collapsible section on the task and person pages: caret, title, optional extra text
  // (e.g. "1/3") and actions on the right.
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  let {
    title,
    extra = '',
    open = $bindable(true),
    actions,
    children,
  }: { title: string; extra?: string; open?: boolean; actions?: Snippet; children: Snippet } = $props();
</script>

<section class="sec">
  <div class="head">
    <button class="sh" aria-expanded={open} onclick={() => (open = !open)}>
      <span class="chev"><Icon name={open ? 'chevronDown' : 'chevronRight'} size={10} stroke={3} /></span>
      <span>{title}</span>
      {#if extra}<span>{extra}</span>{/if}
    </button>
    {#if actions && open}<span class="acts">{@render actions()}</span>{/if}
  </div>
  {#if open}
    <div class="body">{@render children()}</div>
  {/if}
</section>

<style>
  .sec {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .head {
    display: flex;
    align-items: center;
    margin: 0 -16px 4px;
  }

  .head:hover {
    background: var(--c-hover);
  }

  .sh {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 16px;
    border: 0;
    background: transparent;
    font: 400 13px/18px var(--font-ui);
    color: var(--c-muted-2);
    text-align: left;
    cursor: pointer;
  }

  .head:hover .sh {
    color: var(--text);
  }

  .chev {
    display: grid;
    color: var(--c-muted-2);
  }

  .acts {
    display: flex;
    align-items: center;
    gap: 2px;
    padding-right: 10px;
  }

  .body {
    display: flex;
    flex-direction: column;
    padding-left: 16px;
  }
</style>
