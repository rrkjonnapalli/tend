<script lang="ts">
  // The violet timer pill in the bottom bar (soonest countdown, +N more) and its popover.
  import Icon from './Icon.svelte';
  import Popover from './Popover.svelte';
  import TimerRow from './TimerRow.svelte';
  import { timers } from '#lib/stores/timers.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { countdown } from '#lib/util/timer-parse.ts';
  import { label as keyLabel } from '#lib/util/keys.ts';

  let chip = $state<HTMLButtonElement>();
  const next = $derived(timers.list[0]);
  const open = $derived(ui.popup === 'timers' && timers.list.length > 0);
  const more = $derived(timers.list.length - 1);
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
    {#if more > 0}<span class="more">+{more}</span>{/if}
  </button>
{/if}

{#if open}
  <Popover anchor={chip} placement="top-end" offset={8} label="Timers" onclose={ui.closePopup}>
    <div class="pop">
      <div class="head">
        <span>Timers</span>
        <button class="icon-btn sm x" aria-label="Close" title="Close" onclick={ui.closePopup}><Icon name="close" size={12} stroke={2.25} /></button>
      </div>
      {#each timers.list as timer (timer.id)}
        <TimerRow {timer} />
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
    color: var(--c-current-ink);
    font: var(--text-count-sm);
    cursor: pointer;
  }

  .chip:hover {
    background: color-mix(in srgb, var(--c-nav-active-soft) 85%, var(--c-nav-active));
  }

  .more {
    font: var(--text-tiny);
    color: var(--c-muted-2);
  }

  .pop {
    width: 320px;
    display: flex;
    flex-direction: column;
    font: var(--text-caption);
  }

  .head {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px 8px;
    font: var(--text-subsection);
  }

  .x {
    margin-left: auto;
  }

  .foot {
    padding: 10px 12px;
    border-top: 1px solid var(--c-chip);
    color: var(--c-muted-2);
    font-size: 12px;
  }
</style>
