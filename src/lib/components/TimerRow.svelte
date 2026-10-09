<script lang="ts">
  // One running timer in the timers popover: title, countdown, +1m and cancel.
  import type { Timer } from '#lib/types/timer.type.ts';
  import Icon from './Icon.svelte';
  import { timers } from '#lib/stores/timers.svelte.ts';
  import { countdown } from '#lib/util/timer-parse.ts';
  import { timerName } from '#lib/util/timer-text.ts';

  let { timer }: { timer: Timer } = $props();
</script>

<div class="row">
  <span class="tt">{timerName(timer)}</span>
  <span class="left">{countdown(timer.endsAt - timers.now)}</span>
  <button class="plus" onclick={() => timers.addMinute(timer.id)}>+1m</button>
  <button class="icon-btn sm danger" aria-label="Cancel timer" title="Cancel" onclick={() => timers.cancel(timer.id)}>
    <Icon name="close" size={12} stroke={2.25} />
  </button>
</div>

<style>
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
    font: var(--text-count);
  }

  .plus {
    height: 24px;
    padding: 0 8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--primary);
    font: var(--text-meta);
    cursor: pointer;
  }

  .plus:hover {
    background: var(--primary-soft);
  }
</style>
