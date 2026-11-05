<script lang="ts">
  // The phone's + (MD-Quick, MD-Quick-Task): Note · Folder in Notes, Task · Folder in Tasks, with
  // the current folder filled in. A note opens ready to type; a task or folder stays in the list.
  // Names become slugs (the name rule); a task title is free text.
  import { goto } from '$app/navigation';
  import Sheet from './Sheet.svelte';
  import type { QuickSheetKind, Tree } from '#lib/types/ui.type.ts';
  import ChoicePicker from '../ChoicePicker.svelte';
  import { focusOnMount } from '#lib/actions/focus.action.ts';
  import { notes } from '#lib/stores/notes.svelte.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { join, nameError, slug } from '#lib/util/names.ts';
  import { routes } from '#lib/util/routes.ts';
  import { messageOf } from '#lib/errors/app-error.ts';

  let { tree, folder, rootName, onclose }: { tree: Tree; folder: string; rootName: string; onclose: () => void } = $props();

  // svelte-ignore state_referenced_locally
  let target = $state(folder);
  let kind = $state<QuickSheetKind>('leaf');
  let name = $state('');
  let failure = $state('');

  const leafLabel = $derived(tree === 'notes' ? 'Note' : 'Task');
  const title = $derived(kind === 'folder' ? 'New folder' : `New ${leafLabel.toLowerCase()}`);
  const folders = $derived(['', ...(tree === 'notes' ? notes.folders : tasks.folders).filter((f) => f !== '')].sort());
  const isTask = $derived(tree === 'tasks' && kind === 'leaf');
  const canCreate = $derived(isTask ? name.trim() !== '' : nameError(name) === '');

  const labelOf = (path: string) => (path ? `${rootName}/${path}` : rootName);
  const hint = $derived(hintOf(name));

  /** The name rule, and the name a typed one is saved as when it differs. */
  function hintOf(typed: string) {
    const saved = slug(typed);
    if (!typed || saved === typed) return 'Lowercase letters, digits and _ only.';
    return `Lowercase letters, digits and _ only: saved as ${saved || '…'}`;
  }

  async function create() {
    failure = '';
    try {
      if (kind === 'folder') {
        const path = join(target, slug(name));
        if (tree === 'notes') await notes.createFolder(path);
        else await tasks.createFolder(path);
      } else if (tree === 'notes') {
        const row = await notes.create({ folder: target, title: name.trim() });
        goto(routes.note(row.id, { edit: true }));
      } else {
        await tasks.create({ path: target, title: name.trim() });
      }
      onclose();
    } catch (e) {
      failure = messageOf(e);
    }
  }
</script>

<Sheet {title} action="Create" disabled={!canCreate} onaction={create} {onclose}>
  <div class="p-seg" role="radiogroup" aria-label="What to create">
    {#each [['leaf', leafLabel], ['folder', 'Folder']] as [value, label] (value)}
      <label class="p-choice" class:on={kind === value}>
        <input type="radio" name="quick-kind" {value} checked={kind === value} onchange={() => (kind = value as QuickSheetKind)} />{label}
      </label>
    {/each}
  </div>
  <div class="p-field">
    <span class="p-label">in</span>
    <ChoicePicker value={target} choices={folders.map((path) => ({ id: path, label: labelOf(path) }))} what="Folder" field onpick={(id) => (target = id)} />
  </div>
  <label class="p-field">
    <span class="p-label">{isTask ? 'title' : 'name'}</span>
    <input use:focusOnMount class="p-input" class:mono={!isTask} autocapitalize={isTask ? 'sentences' : 'off'} bind:value={name} />
  </label>
  {#if failure}
    <p class="err" role="alert">{failure}</p>
  {:else if !isTask}
    <p class="hint">{hint}</p>
  {/if}
</Sheet>

<style>

  .hint,
  .err {
    margin: 0;
    font: var(--text-caption);
    color: var(--text-muted);
  }

  .err {
    color: var(--danger);
  }
</style>
