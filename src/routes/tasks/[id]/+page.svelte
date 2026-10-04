<script lang="ts">
  // Task page: breadcrumb (folders → parent tasks), title, fields, then the collapsible sections.
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import Icon from '#lib/components/Icon.svelte';
  import Breadcrumbs, { type Crumb } from '#lib/components/Breadcrumbs.svelte';
  import StatusSelect from '#lib/components/StatusSelect.svelte';
  import StatusDot from '#lib/components/StatusDot.svelte';
  import DateField from '#lib/components/DateField.svelte';
  import TimeField from '#lib/components/TimeField.svelte';
  import AssigneeField from '#lib/components/AssigneeField.svelte';
  import TagEditor from '#lib/components/TagEditor.svelte';
  import Section from '#lib/components/Section.svelte';
  import MarkdownField from '#lib/components/MarkdownField.svelte';
  import EditControls from '#lib/components/EditControls.svelte';
  import Attachments from '#lib/components/Attachments.svelte';
  import Comments from '#lib/components/Comments.svelte';
  import Grip from '#lib/components/Grip.svelte';
  import MoveToDialog from '#lib/components/MoveToDialog.svelte';
  import DeleteDialog from '#lib/components/DeleteDialog.svelte';
  import { tasksApi, type TaskPatch } from '#lib/api/tasks.ts';
  import { messageOf } from '#lib/api/core.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { people } from '#lib/stores/people.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { inMyDay, isDone, isOverdue } from '#lib/util/task-views.ts';
  import { keyBetween } from '#lib/util/order-key.ts';
  import { startReorder } from '#lib/util/reorder.ts';
  import { matches } from '#lib/util/keys.ts';
  import { hasLayers } from '#lib/util/layers.ts';
  import { plural } from '#lib/util/plural.ts';
  import { routes } from '#lib/util/routes.ts';

  people.load();

  const id = $derived(page.params.id!);
  const task = $derived(tasks.byId(id));
  const rootName = $derived(settings.get('tasks.root_name', 'tasks'));
  const wide = $derived(settings.get('tasks.wide', false));

  let description = $state('');
  let tags = $state<string[]>([]);
  let detailFor = $state<string | null>(null);

  $effect(() => {
    const taskId = id;
    tasksApi.get(taskId).then(
      (d) => {
        if (taskId !== id) return;
        description = d.description;
        tags = d.tags;
        detailFor = taskId;
      },
      (e) => toast.show(messageOf(e)),
    );
  });

  async function patch(p: TaskPatch) {
    try {
      await tasks.update(id, p);
    } catch (e) {
      toast.show(messageOf(e));
    }
  }

  // ---- title

  let title = $state('');
  $effect(() => {
    title = task?.title ?? '';
  });

  function saveTitle() {
    const t = title.trim();
    if (!task || t === task.title) return;
    if (!t) title = task.title;
    else patch({ title: t });
  }

  // ---- breadcrumb

  const chain = $derived.by(() => {
    const out = [];
    let p = task?.parentId ? tasks.byId(task.parentId) : undefined;
    while (p) {
      out.unshift(p);
      p = p.parentId ? tasks.byId(p.parentId) : undefined;
    }
    return out;
  });

  const crumbs = $derived.by((): Crumb[] => {
    if (!task) return [];
    const out: Crumb[] = [{ label: rootName, href: routes.tasksFolder('') }];
    const segs = task.path ? task.path.split('/') : [];
    segs.forEach((s, i) => out.push({ label: s, href: routes.tasksFolder(segs.slice(0, i + 1).join('/')) }));
    chain.forEach((t) => out.push({ label: t.title, href: routes.task(t.id) }));
    out.push({ label: task.title });
    return out;
  });

  function back() {
    if (!task) return;
    goto(task.parentId ? routes.task(task.parentId) : routes.tasksFolder(task.path));
  }

  // ---- My day, tags, description

  const mine = $derived(task ? inMyDay(task, tasks.day, tasks.myDay) : false);

  async function setTags(next: string[]) {
    tags = next;
    try {
      tags = await tasksApi.setTags(id, next);
    } catch (e) {
      toast.show(messageOf(e));
    }
  }

  let descField = $state<MarkdownField>();
  let descEditing = $state(false);

  async function saveDescription(text: string) {
    description = text;
    await patch({ description: text });
  }

  // ---- subtasks

  const subs = $derived(tasks.subtasksOf(id));
  let subDraft = $state('');
  let subPreview = $state<string[] | null>(null);
  let liftedSub = $state<string | null>(null);
  const shownSubs = $derived(subPreview ? subPreview.map((k) => subs.find((s) => s.id === k)!).filter(Boolean) : subs);

  async function addSub() {
    const t = subDraft.trim();
    if (!t || !task) return;
    const last = subs.at(-1)?.orderKey ?? null;
    try {
      await tasks.create({ path: task.path, title: t, parentId: id, orderKey: keyBetween(last, null) });
      subDraft = '';
    } catch (e) {
      toast.show(messageOf(e));
    }
  }

  function grabSub(e: PointerEvent, subId: string) {
    liftedSub = subId;
    startReorder(e, {
      keys: subs.map((s) => s.id),
      key: subId,
      preview: (p) => {
        subPreview = p;
        if (!p) liftedSub = null;
      },
      drop: (order, i) => {
        const key = (k: string | undefined) => (k ? tasks.byId(k)!.orderKey : null);
        tasks.update(subId, { orderKey: keyBetween(key(order[i - 1]), key(order[i + 1])) }).catch((err) => toast.show(messageOf(err)));
      },
    });
  }

  // ---- counts shown in section headers

  let commentCount = $state(0);

  // ---- move and delete

  let moving = $state(false);
  let deleting = $state(false);

  async function moveTo(target: string) {
    moving = false;
    try {
      await tasks.move(id, target);
      toast.show(`Moved to ${target || rootName}/`);
    } catch (e) {
      toast.show(messageOf(e));
    }
  }

  async function remove() {
    if (!task) return;
    deleting = false;
    const to = task.parentId ? routes.task(task.parentId) : routes.tasksFolder(task.path);
    await tasks.remove(id);
    toast.show(`Deleted ${task.title}`);
    goto(to, { replace: true });
  }

  function onkeydown(e: KeyboardEvent) {
    if (hasLayers() || !task || task.parentId) return;
    if (matches(e, 'mod+shift+m')) {
      e.preventDefault();
      moving = true;
    }
  }
</script>

<svelte:window {onkeydown} />

<main class="main">
  {#if task}
    <article class="page">
      <div class="bar caption">
        <button class="icon-btn sm back" aria-label="Back" title="Back" onclick={back}><Icon name="back" size={13} /></button>
        <Breadcrumbs {crumbs} />
        <span class="grow"></span>
        {#if !task.parentId}
          <button class="icon-btn" aria-label="Move to…" title="Move to…" onclick={() => (moving = true)}><Icon name="folder" size={14} /></button>
        {/if}
        <button class="icon-btn" aria-label="Delete task" title="Delete task" onclick={() => (deleting = true)}><Icon name="trash" size={14} /></button>
        <button
          class="icon-btn"
          aria-label={wide ? 'Narrow width' : 'Full width'}
          title={wide ? 'Narrow width' : 'Full width'}
          onclick={() => settings.set('tasks.wide', !wide)}>
          <Icon name={wide ? 'narrow' : 'widen'} size={14} />
        </button>
      </div>

      <div class="body" style:max-width={wide ? 'none' : 'var(--content-w)'}>
        <input
          class="title heading"
          aria-label="Title"
          bind:value={title}
          onblur={saveTitle}
          onkeydown={(e) => e.key === 'Enter' && (e.currentTarget as HTMLInputElement).blur()} />

        <div class="props">
          <span class="pl">Status</span>
          <div class="pv"><StatusSelect value={task.status} onchange={(status) => patch({ status })} /></div>
          <span class="pl">Due</span>
          <div class="pv">
            <DateField value={task.dueDate} warn={isOverdue(task, tasks.day)} onchange={(dueDate) => patch(dueDate ? { dueDate } : { dueDate: null, dueTime: null })} />
            <TimeField value={task.dueTime} onchange={(dueTime) => patch({ dueTime })} />
          </div>
          <span class="pl">My day</span>
          <div class="pv">
            <button class="toggle" class:add={!mine} class:rem={mine} aria-pressed={mine} onclick={() => tasks.setMyDay(id, !mine)}>
              {mine ? '− My day' : '+ My day'}
            </button>
          </div>
          <span class="pl">Assignee</span>
          <div class="pv"><AssigneeField value={task.assigneeId} onchange={(assigneeId) => patch({ assigneeId })} /></div>
          <span class="pl">Tags</span>
          <div class="pv">{#if detailFor === id}<TagEditor {tags} onchange={setTags} addLabel="+" />{/if}</div>
        </div>

        <div class="sections">
          <Section title="Description">
            {#snippet actions()}
              <EditControls editing={descEditing} what="description" onedit={() => descField?.edit()} oncancel={() => descField?.cancel()} onsave={() => descField?.save()} />
            {/snippet}
            {#if detailFor === id}
              <MarkdownField
                bind:this={descField}
                bind:editing={descEditing}
                value={description}
                onsave={saveDescription}
                placeholder="Add a description…"
                empty="No description"
                label="Description" />
            {/if}
          </Section>

          <Section title="Subtasks" extra={tasks.subtaskCount(id)}>
            <label class="add">
              <span class="plus">+</span>
              <input placeholder="add subtask" aria-label="Add subtask" bind:value={subDraft} onkeydown={(e) => e.key === 'Enter' && addSub()} />
            </label>
            <div class="subs">
              {#each shownSubs as s (s.id)}
                <a
                  class="sub sortable"
                  class:lifted={subPreview !== null && liftedSub === s.id}
                  href={routes.task(s.id)}
                  draggable="false"
                  data-key={s.id}
                  onpointerdown={(e) => grabSub(e, s.id)}>
                  <Grip />
                  <StatusDot value={s.status} />
                  <span class:done={isDone(s)}>{s.title}</span>
                </a>
              {/each}
            </div>
          </Section>

          <Section title="Attachments">
            <Attachments owner="task" {id} />
          </Section>

          <Section title="Comments" extra={commentCount ? String(commentCount) : ''}>
            <Comments task={id} onchange={(n) => (commentCount = n)} />
          </Section>
        </div>
      </div>
    </article>
  {/if}
</main>

{#if moving && task}
  <MoveToDialog
    what={task.title}
    folders={['', ...tasks.folders]}
    current={task.path}
    {rootName}
    onmove={moveTo}
    onclose={() => (moving = false)} />
{/if}

{#if deleting && task}
  <DeleteDialog
    title="Delete {task.title}?"
    body={subs.length ? `Its ${plural(subs.length, 'subtask')}, comments and attachment links are deleted too.` : 'Its comments and attachment links are deleted too.'}
    ondelete={remove}
    onclose={() => (deleting = false)} />
{/if}

<style>
  .main {
    flex: 1 1 auto;
    min-width: 0;
    padding: 28px 24px 64px;
    overflow: auto;
  }

  .page {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .bar {
    position: sticky;
    top: -28px;
    z-index: 3;
    display: flex;
    align-items: center;
    gap: 6px;
    min-height: 28px;
    margin: 0 -24px;
    padding: 10px 24px;
    background: var(--c-page);
  }

  .back {
    margin-left: -6px;
  }

  .grow {
    flex: 1;
  }

  .body {
    width: 100%;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 32px;
  }

  .title {
    width: 100%;
    padding: 0;
    border: 0;
    outline: none;
    background: transparent;
    font: var(--text-heading);
    color: var(--text);
  }

  .props {
    display: grid;
    grid-template-columns: 96px minmax(0, 1fr);
    row-gap: 4px;
    align-items: center;
  }

  .pl {
    font: 400 13px/18px var(--font-ui);
    color: var(--c-muted-2);
  }

  .pv {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 32px;
    font: 400 14px/20px var(--font-ui);
  }

  .toggle {
    height: 28px;
    padding: 0 8px;
    margin-left: -8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    font: 400 14px/20px var(--font-ui);
    cursor: pointer;
  }

  .toggle.add {
    color: var(--primary);
  }

  .toggle.add:hover {
    background: var(--primary-soft);
  }

  .toggle.rem {
    color: var(--danger);
  }

  .toggle.rem:hover {
    background: var(--danger-soft);
  }

  .sections {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .add {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 32px;
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

  .subs {
    display: flex;
    flex-direction: column;
  }

  .sub {
    position: relative;
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 32px;
    padding: 0 16px 0 32px;
    margin: 0 -16px 0 -32px;
    font: 400 14px/20px var(--font-ui);
    color: var(--text);
    text-decoration: none;
    user-select: none;
    touch-action: pan-x;
  }

  .sub:hover {
    background: var(--c-hover);
    color: var(--text);
  }

  .done {
    color: var(--c-muted-2);
  }
</style>
