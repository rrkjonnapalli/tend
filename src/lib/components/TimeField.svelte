<script lang="ts">
  // h : mm with an AM/PM toggle. Type digits or use ↑↓. Clearing the hour clears the time.
  import Icon from './Icon.svelte';
  import { joinTime, splitTime } from '#lib/util/dates.ts';

  let { value, onchange }: { value: string | null; onchange: (hhmm: string | null) => void } = $props();

  let h = $state('');
  let m = $state('');
  let pm = $state(true);

  // Follow outside changes (another task opened, value saved elsewhere).
  $effect(() => {
    if (value) {
      const t = splitTime(value);
      h = String(t.h);
      m = String(t.m).padStart(2, '0');
      pm = t.pm;
    } else {
      h = '';
      m = '';
    }
  });

  function emit() {
    const hn = Number(h);
    if (!h) {
      if (value !== null) onchange(null);
      return;
    }
    if (hn < 1 || hn > 12) return;
    const mn = m ? Number(m) : 0;
    if (mn > 59) return;
    const next = joinTime(hn, mn, pm);
    if (next !== value) onchange(next);
  }

  const digits = (s: string) => s.replace(/\D/g, '').slice(0, 2);

  function step(part: 'h' | 'm', by: number) {
    if (part === 'h') {
      const cur = Number(h) || (by > 0 ? 0 : 1);
      h = String(((cur - 1 + by + 12) % 12) + 1);
    } else {
      const cur = Number(m) || 0;
      m = String((cur + by + 60) % 60).padStart(2, '0');
      if (!h) h = '12';
    }
    emit();
  }

  function key(part: 'h' | 'm', e: KeyboardEvent) {
    if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
      e.preventDefault();
      step(part, e.key === 'ArrowUp' ? 1 : -1);
    } else if (e.key === 'Enter') {
      (e.target as HTMLInputElement).blur();
    }
  }

  function select(e: FocusEvent) {
    (e.target as HTMLInputElement).select();
  }
</script>

<span class="fld" role="group" aria-label="Due time">
  <span class="ic"><Icon name="clock" size={13} /></span>
  <input
    class="seg"
    inputmode="numeric"
    maxlength="2"
    placeholder="--"
    aria-label="Hour"
    bind:value={h}
    oninput={() => (h = digits(h))}
    onkeydown={(e) => key('h', e)}
    onblur={emit}
    onfocus={select} />
  <span class="muted">:</span>
  <input
    class="seg"
    inputmode="numeric"
    maxlength="2"
    placeholder="--"
    aria-label="Minute"
    bind:value={m}
    oninput={() => (m = digits(m))}
    onkeydown={(e) => key('m', e)}
    onblur={() => (m && (m = m.padStart(2, '0')), emit())}
    onfocus={select} />
  <button class="ap" aria-label="Toggle AM or PM" onclick={() => ((pm = !pm), emit())}>{pm ? 'PM' : 'AM'}</button>
</span>

<style>
  .fld {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    height: 28px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--c-page);
    font: 400 13px/18px var(--font-ui);
  }

  .fld:focus-within {
    border-color: var(--primary);
  }

  .ic {
    display: grid;
    color: var(--c-muted-2);
    margin-right: 4px;
  }

  .seg {
    width: 20px;
    padding: 0;
    border: 0;
    outline: none;
    background: transparent;
    text-align: center;
    font: 400 13px/18px var(--font-ui);
    border-radius: var(--radius-xs);
  }

  .seg::placeholder {
    color: var(--c-muted-2);
  }

  .seg:focus {
    background: var(--border);
  }

  .ap {
    height: 20px;
    margin-left: 4px;
    padding: 0 4px;
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    font: 400 13px/18px var(--font-ui);
    cursor: pointer;
  }

  .ap:hover,
  .ap:focus-visible {
    background: var(--border);
  }
</style>
