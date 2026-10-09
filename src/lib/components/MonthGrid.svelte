<script lang="ts">
  // One month of days, Monday first. Shows a single day or a range, with an optional
  // live preview while the second end of a range is being picked.
  import { untrack } from 'svelte';
  import Icon from './Icon.svelte';
  import { dayOfMonth, monthGrid, monthTitle, parseISO, today } from '#lib/util/dates.ts';

  let {
    from,
    to = from,
    preview = null,
    marked = new Set<string>(),
    dimPast = false,
    onpick,
    onhover,
  }: {
    /** Selected day, or the start of the selected range. */
    from: string | null;
    to?: string | null;
    /** While picking a range: the first end, so hovering shows the band. */
    preview?: { anchor: string; hover: string } | null;
    /** Days that get a small dot (e.g. days with tasks due). */
    marked?: Set<string>;
    dimPast?: boolean;
    onpick: (day: string) => void;
    onhover?: (day: string) => void;
  } = $props();

  // The grid opens on the selected month; after that the arrows move it.
  const start = parseISO(untrack(() => from) ?? today());
  let year = $state(start.getFullYear());
  let month = $state(start.getMonth());
  const now = today();
  const WEEKDAY_HEADS = ['MO', 'TU', 'WE', 'TH', 'FR', 'SA', 'SU'];

  const cells = $derived(monthGrid(year, month));
  const band = $derived.by(() => {
    if (preview) {
      const [a, b] = [preview.anchor, preview.hover].sort();
      return { a, b };
    }
    return from ? { a: from, b: to ?? from } : null;
  });

  function shift(n: number) {
    const d = new Date(year, month + n, 1);
    year = d.getFullYear();
    month = d.getMonth();
  }

  /** A band is drawn only for a range of more than one day. */
  function inBand(day: string): boolean {
    if (!band || band.a === band.b) return false;
    return day >= band.a && day <= band.b;
  }

  function cellClass(day: string): string {
    if (!band || !inBand(day)) return '';
    if (day === band.a) return 'bstart';
    if (day === band.b) return 'bend';
    return 'band';
  }

  /** The selected day, or either end of the range. */
  const isEdge = (day: string) => band !== null && (day === band.a || day === band.b);
</script>

<div class="month">
  <div class="head">
    <button class="nav" aria-label="Previous month" onclick={() => shift(-1)}><Icon name="chevronLeft" size={12} stroke={2.5} /></button>
    <span>{monthTitle(year, month)}</span>
    <button class="nav" aria-label="Next month" onclick={() => shift(1)}><Icon name="chevronRight" size={12} stroke={2.5} /></button>
  </div>
  <div class="grid">
    {#each WEEKDAY_HEADS as wd (wd)}<span class="wd">{wd}</span>{/each}
    {#each cells as day, i (day ?? `pad${i}`)}
      <span class="cell {day ? cellClass(day) : ''}">
        {#if day}
          <button
            class="dd"
            class:today={day === now}
            class:edge={isEdge(day)}
            class:past={dimPast && day < now}
            aria-label={day}
            onclick={() => onpick(day)}
            onmouseenter={() => onhover?.(day)}>
            {dayOfMonth(day)}
            {#if marked.has(day)}<span class="dot"></span>{/if}
          </button>
        {/if}
      </span>
    {/each}
  </div>
</div>

<style>
  .month {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font: var(--text-label);
  }

  .nav {
    width: 24px;
    height: 24px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-muted);
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .nav:hover {
    background: var(--c-hover);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(7, 32px);
    row-gap: 2px;
  }

  .wd {
    font: var(--text-tiny);
    color: var(--c-muted-2);
    text-align: center;
    padding-bottom: 4px;
    letter-spacing: 0.04em;
  }

  .cell {
    position: relative;
    height: 32px;
    display: grid;
    place-items: center;
  }

  .cell.band {
    background: var(--c-hover);
  }

  .cell.bstart {
    background: linear-gradient(90deg, transparent 50%, var(--c-hover) 50%);
  }

  .cell.bend {
    background: linear-gradient(90deg, var(--c-hover) 50%, transparent 50%);
  }

  .dd {
    position: relative;
    width: 30px;
    height: 30px;
    padding: 0;
    border: 0;
    border-radius: 15px;
    background: transparent;
    color: var(--text);
    font: var(--text-caption);
    cursor: pointer;
  }

  .dd:hover {
    background: var(--border);
  }

  .dd.past {
    color: var(--c-faint);
  }

  .dd.today {
    color: var(--primary);
    font-weight: 700;
  }

  .dd.edge {
    background: var(--text);
    color: var(--c-page);
    font-weight: 700;
  }

  .dot {
    position: absolute;
    left: 50%;
    bottom: 3px;
    width: 3px;
    height: 3px;
    margin-left: -1.5px;
    border-radius: 2px;
    background: var(--c-muted-2);
  }

  .edge .dot {
    background: var(--c-page);
  }
</style>
