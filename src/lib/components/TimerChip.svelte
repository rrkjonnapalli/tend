<script lang="ts">
  // The violet timer pill in the bottom bar (soonest countdown, +N more) and its popover.
  import Icon from './Icon.svelte';
  import Popover from './Popover.svelte';
  import { timers } from '#lib/stores/timers.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { countdown } from '#lib/util/timer-parse.ts';
  import { label as keyLabel } from '#lib/util/keys.ts';

  let chip = $state<HTMLButtonElement>();
  const next = $derived(timers.list[0]);
  const open = $derived(ui.popup === 'timers' && timers.list.length > 0);
</script>

{#if next}
  <button
    bind:this={chip}
    class="chip"
    aria-label="Running timers"
    title="Timers"
    aria-haspopup="dialog"
    aria-expanded={open}
    onclick={() => ui.toggle('timers')}>
    <Icon name="timer" size={14} />
    <span>{countdown(next.endsAt - timers.now)}</span>
    {#if timers.list.length > 1}<span class="more">+{timers.list.length - 1}</span>{/if}
  </button>
{/if}

{#if open}
  <Popover anchor={chip} placement="top-end" offset={8} label="Timers" onclose={() => (ui.popup = null)}>
    <div class="pop">
      <div class="head">
        <span>Timers</span>
        <button class="icon-btn sm x" aria-label="Close" title="Close" onclick={() => (ui.popup = null)}><Icon name="close" size={12} stroke={2.25} /></button>
      </div>
      {#each timers.list as t (t.id)}
        <div class="row">
          <span class="tt">{t.title || 'Timer'}</span>
          <span class="left">{countdown(t.endsAt - timers.now)}</span>
          <button class="plus" onclick={() => timers.addMinute(t.id)}>+1m</button>
          <button class="icon-btn sm danger" aria-label="Cancel timer" title="Cancel" onclick={() => timers.cancel(t.id)}>
            <Icon name="close" size={12} stroke={2.25} />
          </button>
        </div>
      {/each}
      <div class="foot">{keyLabel('mod+t')} starts a new timer</div>
    </div>
  </Popover>
{/if}

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 26px;
    padding: 0 8px;
    border: 0;
    border-radius: 13px;
    background: var(--c-nav-active-soft);
    color: var(--c-nav-active);
    font: 500 12px/16px var(--font-mono);
    cursor: pointer;
  }

  .chip:hover {
    background: color-mix(in srgb, var(--c-nav-active-soft) 85%, var(--c-nav-active));
  }

  .more {
    font: 400 11px/16px var(--font-ui);
    color: var(--c-muted-2);
  }

  .pop {
    width: 320px;
    display: flex;
    flex-direction: column;
    font: 400 13px/18px var(--font-ui);
  }

  .head {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px 8px;
    font: 500 14px/20px var(--font-heading);
  }

  .x {
    margin-left: auto;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 44px;
    padding: 0 8px 0 12px;
    border-top: 1px solid var(--c-chip);
  }

  .tt {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 14px;
  }

  .left {
    font: 500 14px/20px var(--font-mono);
  }

  .plus {
    height: 24px;
    padding: 0 8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--primary);
    font: 400 12px/16px var(--font-ui);
    cursor: pointer;
  }

  .plus:hover {
    background: var(--primary-soft);
  }

  .foot {
    padding: 10px 12px;
    border-top: 1px solid var(--c-chip);
    color: var(--c-muted-2);
    font-size: 12px;
  }
</style>
