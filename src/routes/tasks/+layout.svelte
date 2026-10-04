<script lang="ts">
  import type { Snippet } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import Explorer, { type ExplorerItem } from '#lib/components/Explorer.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { isUnder, join, parentOf } from '#lib/util/names.ts';
  import { plural } from '#lib/util/plural.ts';
  import { routes } from '#lib/util/routes.ts';
  import { nav } from '#lib/stores/history.svelte.ts';

  let { children }: { children: Snippet } = $props();

  tasks.load();

  const rootName = $derived(settings.get('tasks.root_name', 'tasks'));
  const taskId = $derived(page.params.id ?? null);
  const folder = $derived(page.url.searchParams.get('f') ?? '');
  // On a task page the explorer highlights the task's folder.
  const selected = $derived<ExplorerItem>({ kind: 'folder', path: taskId ? (tasks.byId(taskId)?.path ?? '') : folder });

  function follow(oldPath: string, newPath: string) {
    if (!taskId && isUnder(folder, oldPath)) nav.replace(routes.tasksFolder(newPath + folder.slice(oldPath.length)));
  }

  const actions = {
    open: (item: ExplorerItem) => item.kind === 'folder' && goto(routes.tasksFolder(item.path)),
    newItem: (f: string) => goto(`${routes.tasksFolder(f)}${f ? '&' : '?'}add=1`),
    createFolder: (parent: string, name: string) => tasks.createFolder(join(parent, name)),
    rename: async (item: ExplorerItem, name: string) => {
      if (item.kind !== 'folder') return;
      follow(item.path, await tasks.renameFolder(item.path, name));
    },
    move: async (item: ExplorerItem, target: string) => {
      if (item.kind !== 'folder') return;
      follow(item.path, await tasks.moveFolder(item.path, target));
    },
    remove: async (item: ExplorerItem) => {
      if (item.kind !== 'folder') return;
      await tasks.removeFolder(item.path);
      if (!taskId && isUnder(folder, item.path)) nav.replace(routes.tasksFolder(parentOf(item.path)));
    },
    reload: () => tasks.load(),
    contents: (path: string) => {
      const n = tasks.tasks.filter((t) => isUnder(t.path, path)).length;
      const f = tasks.folders.filter((x) => x !== path && isUnder(x, path)).length;
      if (!n && !f) return 'The folder is empty.';
      return `It contains ${plural(n, 'task')}${f ? ` and ${plural(f, 'folder')}` : ''}. Everything inside is deleted.`;
    },
    renameRoot: (name: string) => settings.set('tasks.root_name', name),
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
