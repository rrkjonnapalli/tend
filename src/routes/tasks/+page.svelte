<script lang="ts">
  // Task folder list: views (My day · All · Overdue · Upcoming · date range), then the rows.
  import { settingDefaults, settingKeys } from '#lib/config/settings.config.ts';
  import type { DayRange } from '#lib/types/date.type.ts';
  import type { DroppedOrder } from '#lib/types/reorder.type.ts';
  import type { Task, NewTask } from '#lib/types/task.type.ts';
  import type { LastView, View } from '#lib/types/task-view.type.ts';
  import { onDestroy } from 'svelte';
  import { page } from '$app/state';
  import Breadcrumbs from '#lib/components/Breadcrumbs.svelte';
  import PageBar from '#lib/components/PageBar.svelte';
  import ListTitle from '#lib/components/ListTitle.svelte';
  import ColumnHeader from '#lib/components/ColumnHeader.svelte';
  import SearchLine from '#lib/components/SearchLine.svelte';
  import AddLine from '#lib/components/AddLine.svelte';
  import { rowNav } from '#lib/actions/row-nav.action.ts';
  import { searchNeedle, textMatches } from '#lib/util/search.ts';
  import DateField from '#lib/components/DateField.svelte';
  import TaskRow from '#lib/components/TaskRow.svelte';
  import TaskBoard from '#lib/components/TaskBoard.svelte';
  import Icon from '#lib/components/Icon.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { people } from '#lib/stores/people.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { defaultView, isDone, isManual, viewCounts, viewTasks } from '#lib/util/task-views.ts';
  import { droppedKey, inPreviewOrder } from '#lib/util/task-order.ts';
  import { folderCrumbs, folderParam, TASK_PARAMS, tasksURL } from '#lib/util/task-urls.ts';
  import { startReorder } from '#lib/util/reorder.ts';
  import { plural } from '#lib/util/plural.ts';
  import { nav } from '#lib/stores/history.svelte.ts';

  // Assignee badges need the people list.
  people.load();

  const VIEWS: [View, string][] = [
    ['myday', 'My day'],
    ['all', 'All'],
    ['overdue', 'Overdue'],
    ['upcoming', 'Upcoming'],
  ];
  /** Views with an "add task" field; a task added in My day is due today and joins today's My day. */
  const ADD_VIEWS: View[] = ['myday', 'all'];
  /** Titles over the list columns: dot · title | edit · My day · count · due · assignee · folder. */
  const COLUMNS = ['', 'Title', '', 'My day', 'Subs', 'Due', 'Who', 'Folder'];
  const EMPTY: Record<View, string> = {
    all: 'No tasks here yet',
    overdue: 'Nothing overdue',
    myday: 'Nothing for today yet',
    upcoming: 'Nothing coming up',
    range: 'Nothing due on these days',
  };

  const params = $derived(page.url.searchParams);
  const folder = $derived(params.get(TASK_PARAMS.folder) ?? '');
  const lastView = $derived(settings.get<LastView | null>(settingKeys.tasksLastView, null));
  const view = $derived((params.get(TASK_PARAMS.view) as View | null) ?? defaultView({ last: lastView, today: tasks.day }));
  const range = $derived.by((): DayRange => {
    const from = params.get(TASK_PARAMS.from) ?? tasks.day;
    const to = params.get(TASK_PARAMS.to) ?? from;
    return [from, to];
  });
  const rootName = $derived(settings.get(settingKeys.tasksRootName, settingDefaults.tasksRootName));
  const board = $derived(settings.get<string>(settingKeys.tasksLayout, settingDefaults.tasksLayout) === 'board');

  const hideDone = $derived(settings.get<boolean>(settingKeys.tasksHideDone, settingDefaults.tasksHideDone));
  const ctx = $derived({ today: tasks.day, myDay: tasks.myDay, range, hideDone });
  let query = $state('');
  const needle = $derived(searchNeedle(query));
  const inView = $derived(viewTasks({ tasks: tasks.tasks, folder, view, ctx }));
  const rows = $derived(inView.filter((t) => textMatches([t.title, t.path], needle)));
  const counts = $derived(viewCounts({ tasks: tasks.tasks, folder, ctx }));
  // Drag order is off while searching: a filtered list hides the neighbours a drop lands between.
  const manual = $derived(isManual(view) && !needle);
  const dueDays = $derived(new Set(tasks.tasks.flatMap((t) => (t.dueDate ? [t.dueDate] : []))));
  const crumbs = $derived(folderCrumbs({ rootName, path: folder }));

  function setView(v: View, extra: Record<string, string> = {}) {
    settings.set(settingKeys.tasksLastView, { view: v, day: tasks.day } satisfies LastView);
    nav.replace(tasksURL({ ...folderParam(folder), [TASK_PARAMS.view]: v, ...extra }));
  }

  function setRange([from, to]: DayRange) {
    setView('range', { [TASK_PARAMS.from]: from, [TASK_PARAMS.to]: to });
  }

  // ---- reorder

  let preview = $state<string[] | null>(null);
  let lifted = $state<string | null>(null);
  const shown = $derived(inPreviewOrder(rows, preview));
  let stopReorder: (() => void) | null = null;

  onDestroy(() => stopReorder?.());

  /** Open and done tasks each move only among themselves. */
  function groupBounds(id: string) {
    const done = isDone(rows.find((t) => t.id === id)!);
    const sameGroup = (t: Task) => isDone(t) === done;
    const min = rows.findIndex(sameGroup);
    const size = rows.filter(sameGroup).length;
    return { min, max: min + size - 1 };
  }

  function showPreview(p: string[] | null) {
    preview = p;
    if (!p) lifted = null;
  }

  async function saveDrop(id: string, { order, index }: DroppedOrder) {
    await tasks.update(id, { orderKey: droppedKey({ order, index, tasks: tasks.tasks }) });
  }

  function grab(e: PointerEvent, id: string) {
    // The My day and status buttons keep their own clicks.
    const onRowButton = (e.target as HTMLElement).closest('.mini, [data-no-drag]');
    if (!manual || onRowButton) return;
    lifted = id;
    stopReorder = startReorder(e, {
      keys: rows.map((t) => t.id),
      key: id,
      ...groupBounds(id),
      preview: showPreview,
      drop: (order, index) => saveDrop(id, { order, index }),
    });
  }

  // ---- add task

  let draft = $state('');
  let addInput = $state<HTMLInputElement>();

  $effect(() => {
    if (params.get(TASK_PARAMS.add) === '1' && addInput) addInput.focus();
  });

  const canAdd = $derived(ADD_VIEWS.includes(view));

  async function add() {
    const title = draft.trim();
    if (!title) return;
    const inMyDay = view === 'myday';
    const input: NewTask = { path: folder, title };
    if (inMyDay) input.dueDate = tasks.day;
    const created = await tasks.create(input);
    draft = '';
    if (inMyDay) await tasks.setMyDay(created.id, true);
  }
</script>

<main class="list-page" use:rowNav>
  <PageBar><Breadcrumbs {crumbs} /></PageBar>
  <ListTitle title={crumbs[crumbs.length - 1].label} meta={plural(counts.all, 'task')} />
  <SearchLine bind:value={query} label="Search tasks" placeholder="Search tasks" />

  <nav class="views" aria-label="Views">
    {#each VIEWS as [v, label], i (v)}
      {#if i > 0}<span class="sep" aria-hidden="true">·</span>{/if}
      <button class="vbtn" class:on={view === v} aria-pressed={view === v} onclick={() => setView(v)}>
        {label}<span class="vc">{counts[v]}</span>
      </button>
    {/each}
    <span class="sep" aria-hidden="true">·</span>
    <DateField mode="range" {range} active={view === 'range'} marked={dueDays} onrange={setRange} />
    <button
      class="icon-btn hide"
      class:on={hideDone}
      aria-label="Hide done tasks"
      aria-pressed={hideDone}
      title={hideDone ? 'Show done tasks' : 'Hide done tasks'}
      onclick={() => settings.set(settingKeys.tasksHideDone, !hideDone)}>
      <Icon name="doneHidden" size={14} />
    </button>
    <span class="lay" role="group" aria-label="Layout">
      <button class="icon-btn" class:on={!board} aria-label="List" aria-pressed={!board} title="List" onclick={() => settings.set(settingKeys.tasksLayout, 'list')}>
        <Icon name="list" size={14} />
      </button>
      <button class="icon-btn" class:on={board} aria-label="Board" aria-pressed={board} title="Board" onclick={() => settings.set(settingKeys.tasksLayout, 'board')}>
        <Icon name="board" size={14} />
      </button>
    </span>
  </nav>

  {#if board}
    <div class="boardwrap"><TaskBoard items={rows} {folder} /></div>
  {:else}
    {#if canAdd}
      <AddLine bind:value={draft} bind:input={addInput} label="Add task" placeholder="add task" onadd={add} />
    {/if}
    <ColumnHeader template="var(--grid-task)" columns={COLUMNS} />
    <div class="list-scroll">
      <div class="rows" role="list">
        {#each shown as task (task.id)}
          <TaskRow
            {task}
            {folder}
            sortable={manual}
            lifted={lifted === task.id && preview !== null}
            onpointerdown={(e) => grab(e, task.id)} />
        {/each}
        {#if shown.length === 0}<span class="list-empty">{needle ? 'No tasks match' : EMPTY[view]}</span>{/if}
      </div>
    </div>
  {/if}
</main>

<style>
  /* The board fills the rest of the window; its columns scroll on their own. */
  .boardwrap {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    padding-bottom: 12px;
  }

  /* Hide done, then the layout switch, at the right end of the views line. */
  .hide {
    margin-left: auto;
    margin-right: var(--space-2);
  }

  .lay {
    display: inline-flex;
    align-items: center;
    gap: 2px;
  }

  /* The filter line under the bar. */
  .views {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    min-height: 36px;
    margin-bottom: 12px;
    flex-wrap: wrap;
    gap: 4px 8px;
    font: var(--text-ui);
  }

  .vbtn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 28px;
    padding: 2px 2px 0;
    border: 0;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--c-muted-2);
    font: var(--text-ui);
    cursor: pointer;
  }

  .vbtn:hover {
    color: var(--text);
  }

  .vbtn.on {
    color: var(--text);
    border-bottom-color: var(--primary);
  }

  .vc {
    font: var(--text-meta);
    color: var(--c-muted-2);
  }

  .sep {
    color: var(--c-muted-2);
    line-height: 28px;
  }

  .rows {
    display: flex;
    flex-direction: column;
  }
</style>
