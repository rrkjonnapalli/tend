<script lang="ts">
  import { settingKeys } from '#lib/config/settings.config.ts';
  // Task page: breadcrumb (folders → parent tasks), title, fields, then the collapsible sections.
  import type { Crumb } from '#lib/types/breadcrumb.type.ts';
  import type { Task, TaskPatch } from '#lib/types/task.type.ts';
  import { page } from '$app/state';
  import Icon from '#lib/components/Icon.svelte';
  import Breadcrumbs from '#lib/components/Breadcrumbs.svelte';
  import PageBar from '#lib/components/PageBar.svelte';
  import TaskTitle from '#lib/components/TaskTitle.svelte';
  import TaskFields from '#lib/components/TaskFields.svelte';
  import TaskSubtasks from '#lib/components/TaskSubtasks.svelte';
  import Section from '#lib/components/Section.svelte';
  import MarkdownField from '#lib/components/MarkdownField.svelte';
  import EditControls from '#lib/components/EditControls.svelte';
  import Attachments from '#lib/components/Attachments.svelte';
  import Comments from '#lib/components/Comments.svelte';
  import MoveToDialog from '#lib/components/MoveToDialog.svelte';
  import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
  import DeletedDialog from '#lib/components/DeletedDialog.svelte';
  import { useDeleted } from '#lib/hooks/deleted.hook.svelte.ts';
  import { tasksAPI } from '#lib/api/tasks.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { people } from '#lib/stores/people.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { matches } from '#lib/util/keys.ts';
  import { pageActions } from '#lib/stores/page-actions.svelte.ts';
  import { hasLayers } from '#lib/util/layers.ts';
  import { plural } from '#lib/util/plural.ts';
  import { routes } from '#lib/util/routes.ts';
  import { folderCrumbs } from '#lib/util/task-urls.ts';
  import { nav } from '#lib/stores/history.svelte.ts';

  people.load();

  const id = $derived(page.params.id!);
  const live = $derived(tasks.byId(id));

  // Deleted elsewhere (by sync): the page keeps showing the task and asks whether to keep it.
  const deleted = useDeleted({ id: () => id, live: () => live });
  /** The open task's labels, still known after the task leaves the store. */
  let lastLabels = $state<string[]>([]);
  $effect(() => {
    if (live) lastLabels = tasks.labelIds.get(live.id) ?? [];
  });

  /** The live task, or its last copy while the "keep it?" question is open. */
  const task = $derived.by(() => {
    if (live) return live;
    if (deleted.gone && deleted.last) return deleted.last;
    return undefined;
  });
  const rootName = $derived(settings.get(settingKeys.tasksRootName, 'tasks'));
  const wide = $derived(settings.get(settingKeys.tasksWide, false));
  const widthLabel = $derived(wide ? 'Narrow width' : 'Full width');

  // ---- detail (description and tags are not in the task list)

  let description = $state('');
  let tags = $state<string[]>([]);
  let detailFor = $state<string | null>(null);
  const detailLoaded = $derived(detailFor === id);

  function loadDetail(taskId: string) {
    tasksAPI.get(taskId).then((d) => {
      // Another task may have opened while this one loaded.
      if (taskId !== id) return;
      description = d.description;
      tags = d.tags;
      detailFor = taskId;
    });
  }

  $effect(() => loadDetail(id));

  async function patch(p: TaskPatch) {
    await tasks.update(id, p);
  }

  async function setTags(next: string[]) {
    tags = next;
    tags = await tasksAPI.setTags({ id, tags: next });
  }

  let descField = $state<MarkdownField>();
  let descEditing = $state(false);

  async function saveDescription(text: string) {
    description = text;
    await patch({ description: text });
  }

  // ---- breadcrumb and back

  /** The parent tasks, outermost first. */
  const ancestors = $derived.by(() => {
    const out: Task[] = [];
    let p = task?.parentId ? tasks.byId(task.parentId) : undefined;
    while (p) {
      out.unshift(p);
      p = p.parentId ? tasks.byId(p.parentId) : undefined;
    }
    return out;
  });

  const crumbs = $derived.by((): Crumb[] => {
    if (!task) return [];
    const parents = ancestors.map((t) => ({ label: t.title, href: routes.task(t.id) }));
    return [...folderCrumbs({ rootName, path: task.path }), ...parents, { label: task.title }];
  });

  /** Where Back goes when there is no history: the parent task, else the task's folder. */
  const upURL = $derived.by(() => {
    if (!task) return routes.tasks;
    if (task.parentId) return routes.task(task.parentId);
    return routes.tasksFolder(task.path);
  });

  function back() {
    if (!task) return;
    nav.back(upURL);
  }

  // ---- move and delete

  let commentCount = $state(0);
  let moving = $state(false);
  let deleting = $state(false);
  const subtaskTotal = $derived(tasks.subtasksOf(id).length);
  const deleteBody = $derived(
    subtaskTotal
      ? `Its ${plural(subtaskTotal, 'subtask')}, comments and attachment links are deleted too.`
      : 'Its comments and attachment links are deleted too.',
  );

  async function moveTo(target: string) {
    moving = false;
    await tasks.move({ id, target });
    toast.show(`Moved to ${target || rootName}/`);
  }

  async function remove() {
    if (!task) return;
    deleting = false;
    const title = task.title;
    const to = upURL;
    await deleted.leave(() => tasks.remove(id));
    toast.show(`Deleted ${title}`);
    nav.back(to);
  }

  async function keep() {
    if (!deleted.last) return;
    // The detail is only known when it loaded for this task.
    const made = await tasks.restore({
      task: deleted.last,
      description: detailLoaded ? description : '',
      tags: detailLoaded ? tags : [],
      labels: lastLabels,
    });
    deleted.settle();
    toast.show('Kept as a new task');
    nav.replace(routes.task(made.id));
  }

  function discard() {
    deleted.settle();
    const folder = deleted.last?.path ?? '';
    nav.replace(routes.tasksFolder(folder));
  }

  // ---- commands and shortcuts

  let fields = $state<TaskFields>();

  const toggleWidth = () => settings.set(settingKeys.tasksWide, !wide);

  $effect(() =>
    pageActions.provide({
      'page.width': toggleWidth,
      'task.status': () => fields?.showStatus(),
      'task.due': () => fields?.showDue(),
      'task.myday': () => tasks.setMyDay(id, true),
    }),
  );

  /** Only top-level tasks live in folders, so only they can move. */
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
      <PageBar onback={back}>
        <Breadcrumbs {crumbs} />
        {#snippet actions()}
        {#if !task.parentId}
          <button class="icon-btn" aria-label="Move to…" title="Move to…" onclick={() => (moving = true)}><Icon name="folder" size={14} /></button>
        {/if}
        <button class="icon-btn" aria-label="Delete task" title="Delete task" onclick={() => (deleting = true)}><Icon name="trash" size={14} /></button>
        <button class="icon-btn" aria-label={widthLabel} title={widthLabel} onclick={toggleWidth}>
          <Icon name={wide ? 'narrow' : 'widen'} size={14} />
        </button>
        {/snippet}
      </PageBar>

      <div class="body" style:max-width={wide ? 'none' : 'var(--content-w)'}>
        <TaskTitle value={task.title} onsave={(title) => patch({ title })} />

        <TaskFields
          bind:this={fields}
          {task}
          {tags}
          {rootName}
          tagsLoaded={detailLoaded}
          onpatch={patch}
          ontags={setTags}
          onmove={() => (moving = true)} />

        <div class="sections">
          <Section title="Description">
            {#snippet actions()}
              <EditControls editing={descEditing} what="description" onedit={() => descField?.edit()} oncancel={() => descField?.cancel()} onsave={() => descField?.save()} />
            {/snippet}
            {#if detailLoaded}
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

          <!-- One level only: a subtask has no subtasks of its own. -->
          {#if !task.parentId}
            <Section title="Subtasks" extra={tasks.subtaskCount(id)}>
              <TaskSubtasks {task} />
            </Section>
          {/if}

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
    kind="task"
    folders={['', ...tasks.folders]}
    current={task.path}
    {rootName}
    onmove={moveTo}
    onclose={() => (moving = false)} />
{/if}

{#if deleted.gone}
  <DeletedDialog kind="task" onkeep={keep} ondiscard={discard} />
{/if}

{#if deleting && task}
  <ConfirmDialog title="Delete {task.title}?" body={deleteBody} confirm="Delete" danger onconfirm={remove} onclose={() => (deleting = false)} />
{/if}

<style>
  .main {
    flex: 1 1 auto;
    min-width: 0;
    padding: 0 24px 64px;
    overflow: auto;
  }

  .page {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }


  .body {
    width: 100%;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 32px;
  }

  .sections {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
</style>
