<script lang="ts">
  // A date button with a month popover. Single mode picks one day (a due date, a date of birth);
  // range mode adds presets and picks one day or two for a range (the tasks nav). A single day is
  // 'yyyy-mm-dd', or 'mm-dd' where the year may be left unknown (`yearOptional`).
  import type { DateFieldProps, DayRange } from '#lib/types/date.type.ts';
  import Icon from './Icon.svelte';
  import Popover from './Popover.svelte';
  import MonthGrid from './MonthGrid.svelte';
  import { pastYears, planYears, shortDate, today } from '#lib/util/dates.ts';
  import { rangePresets } from '#lib/util/date-presets.ts';


  let props: DateFieldProps = $props();

  let open = $state(false);
  let button = $state<HTMLButtonElement>();
  let picking = $state<string | null>(null);
  let hover = $state<string | null>(null);

  const now = today();

  const label = $derived.by(() => {
    if (props.mode === 'range') {
      const [a, b] = props.range;
      if (a !== b) return `${shortDate(a)} – ${shortDate(b)}`;
      return a === now ? `Today, ${shortDate(a)}` : shortDate(a);
    }
    if (!props.value) return emptyLabel;
    return props.format ? props.format(props.value) : shortDate(props.value);
  });

  /** What a single field is for, in its labels: "due date" unless told. */
  const what = $derived(props.mode !== 'range' && props.what ? props.what : 'due date');
  const emptyLabel = $derived(`No ${what}`);

  /** The years the calendar's dropdown lists: back to the earliest for a day whose year may be
   *  unknown (a date of birth), else the years around now (a due date, the tasks range). */
  const years = $derived(props.mode !== 'range' && props.yearOptional ? pastYears() : planYears());

  const presets = rangePresets(now);

  /** Opens the month popover (the "Set due date…" command). */
  export function show() {
    open = true;
  }

  function close() {
    open = false;
    picking = null;
    hover = null;
  }

  function toggle() {
    if (open) close();
    else show();
  }

  /** While picking a range, the hovered day previews its end. */
  function onhover(day: string) {
    if (picking) hover = day;
  }

  function clear() {
    if (props.mode !== 'range') props.onchange(null);
    close();
  }

  const isPresetOn = (r: DayRange, preset: DayRange) => !picking && r[0] === preset[0] && r[1] === preset[1];

  function pick(day: string) {
    if (props.mode !== 'range') {
      props.onchange(day);
      close();
      return;
    }
    if (!picking) {
      picking = day;
      hover = day;
      return;
    }
    const r = [picking, day].sort() as DayRange;
    props.onrange(r);
    close();
  }

  function preset(r: DayRange) {
    if (props.mode === 'range') props.onrange(r);
    close();
  }
</script>

{#if props.mode === 'range'}
  <button
    bind:this={button}
    class="rangebtn"
    class:on={props.active}
    aria-pressed={props.active}
    aria-label="Calendar: pick a date or range"
    aria-haspopup="dialog"
    aria-expanded={open}
    onclick={toggle}>
    <span class="ic"><Icon name="calendar" size={13} /></span>{label}
  </button>
{:else}
  <button
    bind:this={button}
    class="fld"
    class:warn={props.warn}
    class:none={!props.value}
    aria-haspopup="dialog"
    aria-expanded={open}
    onclick={toggle}>
    <span class="ic"><Icon name="calendar" size={13} /></span>{label}
  </button>
{/if}

{#if open}
  <Popover anchor={button} label={props.mode === 'range' ? 'Pick a date or range' : `Pick a ${what}`} onclose={close}>
    <div class="pop">
      {#if props.mode === 'range'}
        {@const r = props.range}
        <div class="presets">
          {#each presets as p (p.name)}
            <button class="pre" class:on={isPresetOn(r, p.range)} data-option onclick={() => preset(p.range)}>{p.name}</button>
          {/each}
        </div>
        <div class="body">
          <MonthGrid
            from={r[0]}
            to={r[1]}
            preview={picking ? { anchor: picking, hover: hover ?? picking } : null}
            marked={props.marked}
            {years}
            onpick={pick}
            {onhover} />
          <div class="foot">{picking ? 'Pick an end date' : 'Pick a day, or two for a range'}</div>
        </div>
      {:else}
        {@const single = props}
        <div class="body">
          <MonthGrid from={single.value} dimPast={single.dimPast ?? true} {years} yearOptional={single.yearOptional} onpick={pick} />
          <button class="clr" aria-label="Clear the {what}" onclick={clear}>Clear</button>
        </div>
      {/if}
    </div>
  </Popover>
{/if}

<style>
  .ic {
    display: grid;
    color: var(--c-muted-2);
  }

  .fld,
  .rangebtn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--c-page);
    color: var(--text);
    font: var(--text-caption);
    cursor: pointer;
  }

  .fld:hover,
  .rangebtn:hover {
    border-color: var(--c-muted-2);
  }

  .fld.warn {
    color: var(--c-overdue);
  }

  .fld.none,
  .rangebtn {
    color: var(--c-muted-2);
  }

  .rangebtn {
    margin-left: 2px;
  }

  .rangebtn.on {
    color: var(--c-current-ink);
    border-color: var(--c-current);
  }

  .pop {
    display: flex;
  }

  .presets {
    display: flex;
    flex-direction: column;
    padding: 8px 0;
    border-right: 1px solid var(--border);
    min-width: 132px;
  }

  .pre {
    height: 30px;
    padding: 0 16px;
    border: 0;
    background: transparent;
    text-align: left;
    font: var(--text-caption);
    color: var(--text-muted);
    cursor: pointer;
  }

  .pre:hover {
    background: var(--c-hover);
    color: var(--text);
  }

  .pre.on {
    color: var(--text);
    font-weight: 700;
  }

  .body {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 14px 14px;
  }

  .foot {
    font: var(--text-meta);
    color: var(--c-muted-2);
  }

  .clr {
    align-self: center;
    height: 24px;
    padding: 0 8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-muted);
    font: var(--text-meta);
    cursor: pointer;
  }

  .clr:hover {
    background: var(--c-hover);
    color: var(--text);
  }
</style>
