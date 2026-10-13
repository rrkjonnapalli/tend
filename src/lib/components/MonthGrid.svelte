<script lang="ts">
  // A calendar in three rows: the year (arrows, and a dropdown of years to jump far), the month
  // (arrows), and that month's days, Monday first. Shows a single day or a range, with an optional
  // live preview while the second end of a range is being picked. Where the year may be left
  // unassigned (a birthday), the days are plain numbers with no weekdays and a pick is 'mm-dd'.
  import { untrack } from 'svelte';
  import ChoicePicker from './ChoicePicker.svelte';
  import Icon from './Icon.svelte';
  import type { Choice } from '#lib/types/ui.type.ts';
  import { calendarStart, clampYear, monthDays, monthGrid, monthName, stepMonth, today } from '#lib/util/dates.ts';

  let {
    from,
    to = from,
    preview = null,
    marked = new Set<string>(),
    dimPast = false,
    years,
    yearOptional = false,
    onpick,
    onhover,
  }: {
    /** Selected day, or the start of the selected range: 'yyyy-mm-dd', or 'mm-dd' without a year. */
    from: string | null;
    to?: string | null;
    /** While picking a range: the first end, so hovering shows the band. */
    preview?: { anchor: string; hover: string } | null;
    /** Days that get a small dot (e.g. days with tasks due). */
    marked?: Set<string>;
    dimPast?: boolean;
    /** The years the year dropdown lists; the arrows reach any year a calendar shows. */
    years: number[];
    /** The year can be left unassigned: it then starts so, and a pick has no year. */
    yearOptional?: boolean;
    onpick: (day: string) => void;
    onhover?: (day: string) => void;
  } = $props();

  const now = today();
  const thisYear = calendarStart(now).year ?? 0;
  const WEEKDAY_HEADS = ['MO', 'TU', 'WE', 'TH', 'FR', 'SA', 'SU'];
  /** The year dropdown's id for no year, and how it reads. */
  const NO_YEAR = '';
  const NO_YEAR_LABEL = 'Unassigned';
  const DAY_AT = -2;

  /** Where the calendar opens: on the selected day; else this month, in this year unless the year
   *  starts unassigned. After that the arrows and the dropdown move it. */
  function opening(): { year: number | null; month: number } {
    const selected = untrack(() => from);
    if (selected) return calendarStart(selected);
    const month = calendarStart(now).month;
    return { year: yearOptional ? null : thisYear, month };
  }

  const start = opening();
  let year = $state<number | null>(start.year);
  let month = $state(start.month);

  /** With a year: the month's days under their weekdays. Without: its days as plain numbers. */
  const cells = $derived(year === null ? monthDays(month) : monthGrid(year, month));
  const dayNumber = (day: string) => Number(day.slice(DAY_AT));

  const yearChoices = $derived.by(() => {
    const listed = years.map((y): Choice => ({ id: String(y), label: String(y) }));
    return yearOptional ? [{ id: NO_YEAR, label: NO_YEAR_LABEL }, ...listed] : listed;
  });

  function pickYear(id: string) {
    year = id === NO_YEAR ? null : Number(id);
  }

  /** An arrow on the year row. From unassigned, either arrow lands on this year. */
  function shiftYear(n: number) {
    year = year === null ? thisYear : clampYear(year + n);
  }

  /** An arrow on the month row; past December or January it carries into the next or last year. */
  function shiftMonth(n: number) {
    const next = stepMonth(month, n);
    month = next.month;
    if (year !== null) year = clampYear(year + next.years);
  }
  const band = $derived.by(() => {
    if (preview) {
      const [a, b] = [preview.anchor, preview.hover].sort();
      return { a, b };
    }
    return from ? { a: from, b: to ?? from } : null;
  });

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
    <button class="nav" aria-label="Previous year" onclick={() => shiftYear(-1)}><Icon name="chevronLeft" size={12} stroke={2.5} /></button>
    <ChoicePicker value={year === null ? NO_YEAR : String(year)} choices={yearChoices} what="Year" max={yearChoices.length} narrow onpick={pickYear} />
    <button class="nav" aria-label="Next year" onclick={() => shiftYear(1)}><Icon name="chevronRight" size={12} stroke={2.5} /></button>
  </div>
  <div class="head">
    <button class="nav" aria-label="Previous month" onclick={() => shiftMonth(-1)}><Icon name="chevronLeft" size={12} stroke={2.5} /></button>
    <span>{monthName(month)}</span>
    <button class="nav" aria-label="Next month" onclick={() => shiftMonth(1)}><Icon name="chevronRight" size={12} stroke={2.5} /></button>
  </div>
  <div class="grid">
    {#if year !== null}
      {#each WEEKDAY_HEADS as wd (wd)}<span class="wd">{wd}</span>{/each}
    {/if}
    {#each cells as day, i (day ?? `pad${i}`)}
      <span class="cell {day ? cellClass(day) : ''}">
        {#if day}
          <button
            class="dd"
            class:today={day === now}
            class:edge={isEdge(day)}
            class:past={dimPast && year !== null && day < now}
            aria-label={day}
            onclick={() => onpick(day)}
            onmouseenter={() => onhover?.(day)}>
            {dayNumber(day)}
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
    font: var(--text-ui);
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
    color: var(--c-muted-2);
  }

  .dd.today {
    color: var(--c-current-ink);
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
