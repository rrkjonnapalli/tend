<script lang="ts">
  import { plainText } from '#lib/actions/plain-text.action.ts';
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
      m = pad2(t.m);
      pm = t.pm;
    } else {
      h = '';
      m = '';
    }
  });

  const HOURS = 12;
  const MINUTES = 60;

  /** Saves the typed time; an incomplete or out-of-range one is left unsaved. */
  function emit() {
    if (!h) {
      if (value !== null) onchange(null);
      return;
    }
    const hour = Number(h);
    if (hour < 1 || hour > HOURS) return;
    const minute = m ? Number(m) : 0;
    if (minute >= MINUTES) return;
    const next = joinTime({ h: hour, m: minute, pm });
    if (next !== value) onchange(next);
  }

  const digits = (s: string) => s.replace(/\D/g, '').slice(0, 2);
  const pad2 = (n: number) => String(n).padStart(2, '0');

  /** `n` brought into 0..size-1, wrapping round at both ends. */
  const wrap = (n: number, size: number) => ((n % size) + size) % size;

  function stepHour(by: number) {
    // From an empty hour, ↑ gives 1 and ↓ gives 12.
    const fallback = by > 0 ? 0 : 1;
    const current = Number(h) || fallback;
    const zeroBased = wrap(current - 1 + by, HOURS);
    h = String(zeroBased + 1);
  }

  function stepMinute(by: number) {
    const current = Number(m) || 0;
    m = pad2(wrap(current + by, MINUTES));
    if (!h) h = '12';
  }

  function step(part: 'h' | 'm', by: number) {
    if (part === 'h') stepHour(by);
    else stepMinute(by);
    emit();
  }

  function finishMinute() {
    if (m) m = m.padStart(2, '0');
    emit();
  }

  function togglePM() {
    pm = !pm;
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
  <input use:plainText
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
  <input use:plainText
    class="seg"
    inputmode="numeric"
    maxlength="2"
    placeholder="--"
    aria-label="Minute"
    bind:value={m}
    oninput={() => (m = digits(m))}
    onkeydown={(e) => key('m', e)}
    onblur={finishMinute}
    onfocus={select} />
  <button class="ap" aria-label="Toggle AM or PM" onclick={togglePM}>{pm ? 'PM' : 'AM'}</button>
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
    font: var(--text-caption);
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
    font: var(--text-caption);
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
    font: var(--text-caption);
    cursor: pointer;
  }

  .ap:hover,
  .ap:focus-visible {
    background: var(--border);
  }
</style>
