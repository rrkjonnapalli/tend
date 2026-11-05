<script lang="ts">
  // Tasks on a phone (MC-Tasks, MC-Tasks-Open): the views (My day ·
  // All · Due · Upcoming, a calendar for the day Due counts up to) and the board switch, then the
  // tasks in this folder and its subfolders. Counts are for this folder.
  import type { LastView, View } from '#lib/types/task-view.type.ts';
  import { page } from '$app/state';
  import Icon from '../Icon.svelte';
  import StatusDot from '../StatusDot.svelte';
  import FolderNav from './FolderNav.svelte';
  import QuickSheet from './QuickSheet.svelte';
  import SelectBar from './SelectBar.svelte';
  import Selection from './Selection.svelte';
  import { longPress } from '#lib/actions/long-press.action.ts';
  import { clip } from '#lib/stores/clip.svelte.ts';
  import PhoneBoard from './PhoneBoard.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { nav } from '#lib/stores/history.svelte.ts';
  import { settingDefaults, settingKeys } from '#lib/config/settings.config.ts';
  import { defaultView, isDone, isOverdue, isView, viewCounts, viewTasks } from '#lib/util/task-views.ts';
  import { folderParam, TASK_PARAMS, tasksURL } from '#lib/util/task-urls.ts';
  import { searchNeedle, textMatches } from '#lib/util/search.ts';
  import { dueLabel, shortDate } from '#lib/util/dates.ts';
  import { routes } from '#lib/util/routes.ts';

  let { folder }: { folder: string } = $props();

  const VIEWS: [View, string][] = [
    ['myday', 'My day'],
    ['all', 'All'],
    ['due', 'Due'],
    ['upcoming', 'Upcoming'],
  ];

  let query = $state('');
  let adding = $state(false);

  const selecting = $derived(clip.selecting?.tree === 'tasks');
  let picker = $state<HTMLInputElement>();

  const params = $derived(page.url.searchParams);
  const rootName = $derived(settings.get(settingKeys.tasksRootName, settingDefaults.tasksRootName));
  const lastView = $derived(settings.get<LastView | null>(settingKeys.tasksLastView, null));
  const asked = $derived(params.get(TASK_PARAMS.view));
  // The phone has no range view; a range from the desktop opens as Due.
  const view = $derived.by((): View => {
    const v = isView(asked) ? asked : defaultView({ last: lastView, today: tasks.day });
    return v === 'range' ? 'due' : v;
  });
  const dueOn = $derived(params.get(TASK_PARAMS.dueOn) ?? tasks.day);
  const board = $derived(settings.get<string>(settingKeys.tasksLayout, settingDefaults.tasksLayout) === 'board');
  const hideDone = $derived(settings.get<boolean>(settingKeys.tasksHideDone, settingDefaults.tasksHideDone));
  const ctx = $derived({ today: tasks.day, myDay: tasks.myDay, range: [tasks.day, tasks.day] as [string, string], hideDone, dueOn });

  const needle = $derived(searchNeedle(query));
  const counts = $derived(viewCounts({ tasks: tasks.tasks, folder, ctx }));
  const rows = $derived(viewTasks({ tasks: tasks.tasks, folder, view, ctx }).filter((t) => textMatches([t.title, t.path], needle)));

  function go(extra: Record<string, string>) {
    nav.replace(tasksURL({ ...folderParam(folder), ...extra }), { keepFocus: true });
  }

  function setView(v: View) {
    settings.set(settingKeys.tasksLastView, { view: v, day: tasks.day } satisfies LastView);
    go({ [TASK_PARAMS.view]: v });
  }

  function pickDay() {
    picker?.showPicker();
  }

  function onPicked() {
    const day = picker?.value;
    if (!day) return;
    settings.set(settingKeys.tasksLastView, { view: 'due', day: tasks.day } satisfies LastView);
    go({ [TASK_PARAMS.view]: 'due', [TASK_PARAMS.dueOn]: day });
  }

  function toggleBoard() {
    settings.set(settingKeys.tasksLayout, board ? 'list' : 'board');
  }
</script>

<main class="phone-list">
  {#if selecting}
    <SelectBar all={rows.map((t) => t.id)} />
  {:else}
    <FolderNav {rootName} {folder} hrefOf={routes.tasksFolder} placeholder="Search tasks" bind:query onadd={() => (adding = true)} />
  {/if}

  <nav class="views" aria-label="Views">
    {#each VIEWS as [value, label] (value)}
      <button class="vt" class:on={view === value} aria-pressed={view === value} onclick={() => setView(value)}>
        {label}<span class="n">{counts[value]}</span>
      </button>
    {/each}
    <button class="ib" class:on={view === 'due' && dueOn !== tasks.day} aria-label="Due by a day" onclick={pickDay}>
      <Icon name="calendar" size={18} />
    </button>
    <input bind:this={picker} class="picker" type="date" tabindex="-1" aria-hidden="true" value={dueOn} onchange={onPicked} />
    <button class="ib end" class:on={board} aria-pressed={board} aria-label="Board" onclick={toggleBoard}>
      <Icon name="board" size={18} />
    </button>
  </nav>

  {#if view === 'due' && dueOn !== tasks.day}
    <p class="due-by">Due by <b>{shortDate(dueOn)}</b> · <button class="link" onclick={() => go({ [TASK_PARAMS.view]: 'due' })}>today</button></p>
  {/if}

  {#if board}
    <PhoneBoard tasks={rows} {folder} />
  {:else}
    {#each rows as t (t.id)}
      {#snippet body()}
        <StatusDot value={t.status} size={9} />
        <span class="tt" class:done={isDone(t)}>{t.title}</span>
        {#if t.dueDate}<span class="due" class:warn={isOverdue(t, tasks.day)}>{dueLabel(t.dueDate)}</span>{/if}
      {/snippet}
      {#if selecting}
        {@const on = clip.selecting?.ids.includes(t.id)}
        <button class="qr" class:sel={on} aria-pressed={on} onclick={() => clip.toggle(t.id)}>
          <span class="check"><Icon name="check" size={14} stroke={3} /></span>{@render body()}
        </button>
      {:else}
        <a class="qr" href={routes.task(t.id)} use:longPress={() => clip.start('tasks', t.id)}>{@render body()}</a>
      {/if}
    {:else}
      <p class="empty">{needle ? 'No tasks match' : 'Nothing here'}</p>
    {/each}
  {/if}
</main>

{#if selecting}
  <Selection
    {folder}
    noun="task"
    asksDelete={settings.asksDelete('task')}
    nameOf={(id) => tasks.byId(id)?.title ?? ''}
    onrename={(id, title) => tasks.update(id, { title })}
    onremove={tasks.remove} />
{/if}

{#if adding}<QuickSheet tree="tasks" {folder} {rootName} onclose={() => (adding = false)} />{/if}

<style>
  .views {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 2px 0 10px;
    white-space: nowrap;
    overflow-x: auto;
    scrollbar-width: none;
  }

  .vt {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 36px;
    padding: 2px 0 0;
    border: 0;
    border-bottom: 2px solid transparent;
    background: transparent;
    font: var(--text-ui);
    font-size: 15px;
    color: var(--c-muted-2);
  }

  .vt.on {
    color: var(--c-current-ink);
    border-bottom-color: var(--c-current);
  }

  .n {
    font: var(--text-meta);
    color: var(--c-muted-2);
  }

  .ib {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    margin: 0 -6px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--c-muted-2);
  }

  .ib.on {
    color: var(--c-current);
  }

  .ib.end {
    margin-left: auto;
  }

  .vt:focus-visible,
  .ib:focus-visible,
  .ib:active {
    outline: none;
    background: var(--c-hover);
  }

  /* The phone's own date picker opens from the calendar button; the field itself stays hidden. */
  .picker {
    position: absolute;
    width: 1px;
    height: 1px;
    opacity: 0;
    pointer-events: none;
  }

  .due-by {
    margin: -4px 0 6px;
    font: var(--text-caption);
    color: var(--c-muted-2);
  }

  .due-by b {
    color: var(--text);
  }

  .link {
    padding: 0;
    border: 0;
    background: transparent;
    font: inherit;
    color: var(--primary);
  }
</style>
