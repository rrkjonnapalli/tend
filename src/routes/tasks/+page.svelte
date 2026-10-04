<script lang="ts">
  // Task folder list: views (All · Overdue · My day · Upcoming · date range), then the rows.
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import Breadcrumbs, { type Crumb } from '#lib/components/Breadcrumbs.svelte';
  import DateField, { type Range } from '#lib/components/DateField.svelte';
  import TaskRow from '#lib/components/TaskRow.svelte';
  import TaskBoard from '#lib/components/TaskBoard.svelte';
  import Icon from '#lib/components/Icon.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { messageOf } from '#lib/api/core.ts';
  import { isDone, isManual, viewCounts, viewTasks, type View } from '#lib/util/task-views.ts';
  import { keyBetween } from '#lib/util/order-key.ts';
  import { startReorder } from '#lib/util/reorder.ts';
  import { baseOf } from '#lib/util/names.ts';
  import { plural } from '#lib/util/plural.ts';
  import { routes } from '#lib/util/routes.ts';

  const VIEWS: [View, string][] = [
    ['all', 'All'],
    ['overdue', 'Overdue'],
    ['myday', 'My day'],
    ['upcoming', 'Upcoming'],
  ];
  const EMPTY: Record<View, string> = {
    all: 'No tasks here yet',
    overdue: 'Nothing overdue',
    myday: 'Nothing for today yet',
    upcoming: 'Nothing coming up',
    range: 'Nothing due on these days',
  };

  const params = $derived(page.url.searchParams);
  const folder = $derived(params.get('f') ?? '');
  const view = $derived((params.get('v') as View) || 'all');
  const range = $derived<Range>([params.get('from') ?? tasks.day, params.get('to') ?? params.get('from') ?? tasks.day]);
  const rootName = $derived(settings.get('tasks.root_name', 'tasks'));
  const board = $derived(settings.get<string>('tasks.layout', 'list') === 'board');

  const ctx = $derived({ today: tasks.day, myDay: tasks.myDay, range });
  const rows = $derived(viewTasks(tasks.tasks, folder, view, ctx));
  const counts = $derived(viewCounts(tasks.tasks, folder, ctx));
  const manual = $derived(isManual(view));
  const dueDays = $derived(new Set(tasks.tasks.flatMap((t) => (t.dueDate ? [t.dueDate] : []))));

  let preview = $state<string[] | null>(null);
  const shown = $derived(preview ? preview.map((id) => rows.find((t) => t.id === id)!).filter(Boolean) : rows);
  let lifted = $state<string | null>(null);

  const crumbs = $derived.by((): Crumb[] => {
    const out: Crumb[] = [{ label: rootName, href: routes.tasksFolder('') }];
    const segs = folder ? folder.split('/') : [];
    segs.forEach((s, i) => out.push({ label: s, href: routes.tasksFolder(segs.slice(0, i + 1).join('/')) }));
    return out;
  });

  function setView(v: View, extra: Record<string, string> = {}) {
    const q = new URLSearchParams({ ...(folder ? { f: folder } : {}), ...(v === 'all' ? {} : { v }), ...extra });
    const s = q.toString();
    goto(`/tasks${s ? `?${s}` : ''}`, { replace: true });
  }

  function grab(e: PointerEvent, id: string) {
    if (!manual || (e.target as HTMLElement).closest('.mini')) return;
    const keys = rows.map((t) => t.id);
    const done = isDone(rows.find((t) => t.id === id)!);
    const group = keys.filter((k) => isDone(rows.find((t) => t.id === k)!) === done);
    const min = keys.indexOf(group[0]);
    lifted = id;
    startReorder(e, {
      keys,
      key: id,
      min,
      max: min + group.length - 1,
      preview: (p) => {
        preview = p;
        if (!p) lifted = null;
      },
      drop: async (order, i) => {
        const key = (k: string | undefined) => (k ? tasks.byId(k)!.orderKey : null);
        try {
          await tasks.update(id, { orderKey: keyBetween(key(order[i - 1]), key(order[i + 1])) });
        } catch (err) {
          toast.show(messageOf(err));
        }
      },
    });
  }

  // ---- add task

  let draft = $state('');
  let addInput = $state<HTMLInputElement>();

  $effect(() => {
    if (params.get('add') === '1' && addInput) addInput.focus();
  });

  async function add() {
    const title = draft.trim();
    if (!title) return;
    try {
      await tasks.create({ path: folder, title });
      draft = '';
    } catch (err) {
      toast.show(messageOf(err));
    }
  }
</script>

<main class="main" class:board>
  <article class="list">
    <div class="head">
      <Breadcrumbs {crumbs} />
      <h1 class="heading">{folder ? baseOf(folder) : rootName}</h1>
      <span class="caption muted">{plural(counts.all, 'task')}</span>
    </div>

    <nav class="views" aria-label="Views">
      {#each VIEWS as [v, label], i (v)}
        {#if i > 0}<span class="sep" aria-hidden="true">·</span>{/if}
        <button class="vbtn" class:on={view === v} aria-pressed={view === v} onclick={() => setView(v)}>
          {label}<span class="vc">{counts[v]}</span>
        </button>
      {/each}
      <span class="sep" aria-hidden="true">·</span>
      <DateField mode="range" {range} active={view === 'range'} marked={dueDays} onrange={([a, b]) => setView('range', { from: a, to: b })} />
      <span class="lay" role="group" aria-label="Layout">
        <button class="lbtn" class:on={!board} aria-label="List" aria-pressed={!board} title="List" onclick={() => settings.set('tasks.layout', 'list')}>
          <Icon name="list" size={14} />
        </button>
        <button class="lbtn" class:on={board} aria-label="Board" aria-pressed={board} title="Board" onclick={() => settings.set('tasks.layout', 'board')}>
          <Icon name="board" size={14} />
        </button>
      </span>
    </nav>

    {#if board}
      <TaskBoard items={rows} {folder} />
    {:else}
    <div class="rows" role="list">
      {#if view === 'all'}
        <label class="add">
          <span class="plus">+</span>
          <input
            bind:this={addInput}
            placeholder="add task"
            aria-label="Add task"
            bind:value={draft}
            onkeydown={(e) => e.key === 'Enter' && add()} />
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
    padding-top: 48px;
    display: flex;
    flex-direction: column;
    gap: 24px;
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

  .head {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .views {
    display: flex;
    align-items: center;
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
