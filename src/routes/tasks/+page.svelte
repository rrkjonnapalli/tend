<script lang="ts">
  // Task folder list: views (My day · All · Overdue · Upcoming · date range), then the rows.
  import { settingKeys } from '#lib/config/settings.config.ts';
  import type { DayRange } from '#lib/types/date.type.ts';
  import type { DroppedOrder } from '#lib/types/reorder.type.ts';
  import type { Task } from '#lib/types/task.type.ts';
  import type { LastView, View } from '#lib/types/task-view.type.ts';
  import { onDestroy } from 'svelte';
  import { page } from '$app/state';
  import Breadcrumbs from '#lib/components/Breadcrumbs.svelte';
  import PageBar from '#lib/components/PageBar.svelte';
  import { focusOnMount } from '#lib/actions/focus.action.ts';
  import DateField from '#lib/components/DateField.svelte';
  import TaskRow from '#lib/components/TaskRow.svelte';
  import TaskBoard from '#lib/components/TaskBoard.svelte';
  import Icon from '#lib/components/Icon.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { defaultView, isDone, isManual, viewCounts, viewTasks } from '#lib/util/task-views.ts';
  import { droppedKey, inPreviewOrder } from '#lib/util/task-order.ts';
  import { folderCrumbs, folderParam, TASK_PARAMS, tasksURL } from '#lib/util/task-urls.ts';
  import { startReorder } from '#lib/util/reorder.ts';
  import { plural } from '#lib/util/plural.ts';
  import { nav } from '#lib/stores/history.svelte.ts';

  const VIEWS: [View, string][] = [
    ['myday', 'My day'],
    ['all', 'All'],
    ['overdue', 'Overdue'],
    ['upcoming', 'Upcoming'],
  ];
  /** Views with an "add task" field; a task added in My day joins today's My day. */
  const ADD_VIEWS: View[] = ['myday', 'all'];
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
  const rootName = $derived(settings.get(settingKeys.tasksRootName, 'tasks'));
  const board = $derived(settings.get<string>(settingKeys.tasksLayout, 'list') === 'board');

  const ctx = $derived({ today: tasks.day, myDay: tasks.myDay, range });
  const rows = $derived(viewTasks({ tasks: tasks.tasks, folder, view, ctx }));
  const counts = $derived(viewCounts({ tasks: tasks.tasks, folder, ctx }));
  const manual = $derived(isManual(view));
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
    const onMyDayButton = (e.target as HTMLElement).closest('.mini');
    if (!manual || onMyDayButton) return;
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
    const created = await tasks.create({ path: folder, title });
    draft = '';
    if (view === 'myday') await tasks.setMyDay(created.id, true);
  }

  function addKeys(e: KeyboardEvent) {
    if (e.key === 'Enter') add();
  }
</script>

<main class="main" class:board>
  <PageBar meta={plural(counts.all, 'task')}><Breadcrumbs {crumbs} /></PageBar>
  <article class="list">

    <nav class="views" aria-label="Views">
      {#each VIEWS as [v, label], i (v)}
        {#if i > 0}<span class="sep" aria-hidden="true">·</span>{/if}
        <button class="vbtn" class:on={view === v} aria-pressed={view === v} onclick={() => setView(v)}>
          {label}<span class="vc">{counts[v]}</span>
        </button>
      {/each}
      <span class="sep" aria-hidden="true">·</span>
      <DateField mode="range" {range} active={view === 'range'} marked={dueDays} onrange={setRange} />
      <span class="lay" role="group" aria-label="Layout">
        <button class="lbtn" class:on={!board} aria-label="List" aria-pressed={!board} title="List" onclick={() => settings.set(settingKeys.tasksLayout, 'list')}>
          <Icon name="list" size={14} />
        </button>
        <button class="lbtn" class:on={board} aria-label="Board" aria-pressed={board} title="Board" onclick={() => settings.set(settingKeys.tasksLayout, 'board')}>
          <Icon name="board" size={14} />
        </button>
      </span>
    </nav>

    {#if board}
      <TaskBoard items={rows} {folder} />
    {:else}
    <div class="rows" role="list">
      {#if canAdd}
        <label class="add">
          <span class="plus">+</span>
          <input
            bind:this={addInput}
            use:focusOnMount
            placeholder="add task"
            aria-label="Add task"
            bind:value={draft}
            onkeydown={addKeys} />
        </label>
      {/if}
      {#each shown as task (task.id)}
        <TaskRow
          {task}
          {folder}
          showParent={view !== 'all'}
          sortable={manual}
          lifted={lifted === task.id && preview !== null}
          onpointerdown={(e) => grab(e, task.id)} />
      {/each}
      {#if shown.length === 0}<span class="caption muted empty">{EMPTY[view]}</span>{/if}
    </div>
    {/if}
  </article>
</main>

<style>
  .main {
    flex: 1 1 auto;
    min-width: 0;
    padding: 0 24px 48px;
    overflow: auto;
  }

  .list {
    display: flex;
    flex-direction: column;
  }

  /* The board fills the window; its columns scroll on their own. */
  .main.board {
    display: flex;
    flex-direction: column;
    padding-bottom: 12px;
  }

  .board .list {
    flex: 1;
    min-height: 0;
  }

  .lay {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    margin-left: auto;
  }

  .lbtn {
    width: 26px;
    height: 26px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--c-muted-2);
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .lbtn:hover {
    color: var(--text);
    background: var(--c-hover);
  }

  .lbtn.on {
    color: var(--text);
    background: var(--c-chip);
  }

  /* The filter line under the bar. */
  .views {
    display: flex;
    align-items: center;
    min-height: 36px;
    margin-bottom: 4px;
    flex-wrap: wrap;
    gap: 4px 8px;
    font: 400 14px/20px var(--font-ui);
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
    font: 400 14px/20px var(--font-ui);
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
    font: 400 12px/16px var(--font-ui);
    color: var(--c-muted-2);
  }

  .sep {
    color: var(--c-muted-2);
    line-height: 28px;
  }

  .rows {
    display: flex;
    flex-direction: column;
    margin-top: -8px;
  }

  .add {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 36px;
    font: 400 14px/20px var(--font-ui);
    color: var(--c-muted-2);
  }

  .plus {
    width: 8px;
    text-align: center;
  }

  .add input {
    flex: 1;
    border: 0;
    outline: none;
    background: transparent;
    font: 400 14px/20px var(--font-ui);
    color: var(--text);
  }

  .add input::placeholder {
    color: var(--c-muted-2);
  }

  .empty {
    padding: 8px 0 0 20px;
  }
</style>
