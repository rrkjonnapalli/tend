<script lang="ts">
  import { settingDefaults, settingKeys } from '#lib/config/settings.config.ts';
  import type { ExplorerActions, ExplorerItem } from '#lib/types/explorer.type.ts';
  import type { Snippet } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import Explorer from '#lib/components/Explorer.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { isUnder, join, parentOf } from '#lib/util/names.ts';
  import { plural } from '#lib/util/plural.ts';
  import { routes } from '#lib/util/routes.ts';
  import { newTaskURL, TASK_PARAMS } from '#lib/util/task-urls.ts';
  import { nav } from '#lib/stores/history.svelte.ts';

  let { children }: { children: Snippet } = $props();

  tasks.load();

  const rootName = $derived(settings.get(settingKeys.tasksRootName, settingDefaults.tasksRootName));
  const taskId = $derived(page.params.id ?? null);
  const folder = $derived(page.url.searchParams.get(TASK_PARAMS.folder) ?? '');

  // On a task page the explorer highlights the task's folder.
  const selected = $derived.by((): ExplorerItem => {
    if (!taskId) return { kind: 'folder', path: folder };
    return { kind: 'folder', path: tasks.byId(taskId)?.path ?? '' };
  });

  /** The folder list on screen is `path` or inside it. */
  const showsFolder = (path: string) => !taskId && isUnder(folder, path);

  /** Keeps the list on a folder that was renamed or moved from `from` to `to`. */
  function follow({ from, to }: { from: string; to: string }) {
    if (!showsFolder(from)) return;
    const rest = folder.slice(from.length);
    nav.replace(routes.tasksFolder(to + rest));
  }

  /** What deleting `path` takes with it, for the delete dialog. */
  function folderContents(path: string): string {
    const taskCount = tasks.tasks.filter((t) => isUnder(t.path, path)).length;
    const folderCount = tasks.folders.filter((x) => x !== path && isUnder(x, path)).length;
    if (!taskCount && !folderCount) return 'The folder is empty.';
    const folders = folderCount ? ` and ${plural(folderCount, 'folder')}` : '';
    return `It contains ${plural(taskCount, 'task')}${folders}. Everything inside is deleted.`;
  }

  const actions: ExplorerActions = {
    open: (item) => {
      if (item.kind === 'folder') goto(routes.tasksFolder(item.path));
    },
    newItem: (f) => goto(newTaskURL(f)),
    createFolder: ({ parent, name }) => tasks.createFolder(join(parent, name)),
    rename: async (item, name) => {
      if (item.kind !== 'folder') return;
      follow({ from: item.path, to: await tasks.renameFolder({ path: item.path, name }) });
    },
    move: async (item, target) => {
      if (item.kind !== 'folder') return;
      follow({ from: item.path, to: await tasks.moveFolder({ path: item.path, target }) });
    },
    remove: async (item) => {
      if (item.kind !== 'folder') return;
      await tasks.removeFolder(item.path);
      if (showsFolder(item.path)) nav.replace(routes.tasksFolder(parentOf(item.path)));
    },
    reload: () => tasks.load(),
    contents: folderContents,
    renameRoot: (name) => settings.set(settingKeys.tasksRootName, name),
  };
</script>

<Explorer id="tasks" {rootName} folders={tasks.folders} {selected} newItemLabel="New task" {actions} />
<div class="pane">{@render children()}</div>

<style>
  .pane {
    flex: 1 1 auto;
    min-width: 0;
    display: flex;
  }
</style>
