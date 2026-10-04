<script lang="ts" module>
  export type Range = [string, string];
</script>

<script lang="ts">
  // A date button with a month popover. Single mode picks one day (due dates); range mode
  // adds presets and picks one day or two for a range (the tasks nav).
  import Icon from './Icon.svelte';
  import Popover from './Popover.svelte';
  import MonthGrid from './MonthGrid.svelte';
  import { addDays, parseIso, iso, shortDate, today } from '#lib/util/dates.ts';

  type Props =
    | { mode?: 'single'; value: string | null; onchange: (day: string | null) => void; warn?: boolean }
    | { mode: 'range'; range: Range; onrange: (r: Range) => void; active?: boolean; marked?: Set<string> };

  let props: Props = $props();

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
    return props.value ? shortDate(props.value) : 'No due date';
  });

  const presets = $derived.by((): [string, Range][] => {
    const dow = (parseIso(now).getDay() + 6) % 7;
    const d = parseIso(now);
    const last = iso(new Date(d.getFullYear(), d.getMonth() + 1, 0));
    return [
      ['Today', [now, now]],
      ['Tomorrow', [addDays(now, 1), addDays(now, 1)]],
      ['This week', [addDays(now, -dow), addDays(now, 6 - dow)]],
      ['Next 7 days', [now, addDays(now, 6)]],
      ['This month', [`${now.slice(0, 8)}01`, last]],
    ];
  });

  function close() {
    open = false;
    picking = null;
    hover = null;
  }

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
    const r = [picking, day].sort() as Range;
    props.onrange(r);
    close();
  }

  function preset(r: Range) {
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
    onclick={() => (open ? close() : (open = true))}>
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
    onclick={() => (open ? close() : (open = true))}>
    <span class="ic"><Icon name="calendar" size={13} /></span>{label}
  </button>
{/if}

{#if open}
  <Popover anchor={button} label={props.mode === 'range' ? 'Pick a date or range' : 'Pick a due date'} onclose={close}>
    <div class="pop">
      {#if props.mode === 'range'}
        {@const r = props.range}
        <div class="presets">
          {#each presets as [name, pr] (name)}
            <button class="pre" class:on={!picking && r[0] === pr[0] && r[1] === pr[1]} onclick={() => preset(pr)}>{name}</button>
          {/each}
        </div>
        <div class="body">
          <MonthGrid
            from={r[0]}
            to={r[1]}
            preview={picking ? { anchor: picking, hover: hover ?? picking } : null}
            marked={props.marked}
            onpick={pick}
            onhover={(d) => picking && (hover = d)} />
          <div class="foot">{picking ? 'Pick an end date' : 'Pick a day, or two for a range'}</div>
        </div>
      {:else}
        {@const single = props}
        <div class="body">
          <MonthGrid from={single.value} dimPast onpick={pick} />
          <button class="clr" onclick={() => (single.onchange(null), close())}>No due date</button>
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
    font: 400 13px/18px var(--font-ui);
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
    color: var(--text);
    border-color: var(--primary);
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
    font: 400 13px/18px var(--font-ui);
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
    font: 400 12px/16px var(--font-ui);
    color: var(--c-muted-2);
  }

  .clr {
    align-self: flex-start;
    height: 24px;
    padding: 0 6px;
    margin-left: -6px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-muted);
    font: 400 12px/16px var(--font-ui);
    cursor: pointer;
  }

  .clr:hover {
    background: var(--c-hover);
    color: var(--text);
  }
</style>
