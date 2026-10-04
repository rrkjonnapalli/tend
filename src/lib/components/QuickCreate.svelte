<script lang="ts">
  // Quick create: ⌘N note · ⌘⇧T task · ⌘⇧N folder (Notes/Tasks) · ⌘T timer.
  import type { Duration } from '#lib/types/timer.type.ts';
  import type { QuickKind } from '#lib/types/ui.type.ts';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import Panel from './Panel.svelte';
  import PathField from './PathField.svelte';
  import { messageOf } from '#lib/api/core.ts';
  import { notes } from '#lib/stores/notes.svelte.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { timers } from '#lib/stores/timers.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { isValidPath, join, nameError } from '#lib/util/names.ts';
  import { nameFromTitle } from '#lib/util/quick-parse.ts';
  import { parseDuration } from '#lib/util/timer-parse.ts';
  import { label as keyLabel } from '#lib/util/keys.ts';
  import { routes } from '#lib/util/routes.ts';

  let { kind, onclose }: { kind: QuickKind; onclose: () => void } = $props();

  const TITLES: Record<QuickKind, string> = { note: 'New note', task: 'New task', folder: 'New folder', timer: 'New timer' };
  const KEYS: Record<QuickKind, string> = { note: 'mod+n', task: 'mod+shift+t', folder: 'mod+shift+n', timer: 'mod+t' };

  // Folder: Notes or Tasks, defaulting to the section you're in.
  let tree = $state<'notes' | 'tasks'>(page.url.pathname.startsWith('/tasks') ? 'tasks' : 'notes');
  const settingKey = $derived(`quick.last_path.${kind}`);
  let path = $state('');
  let name = $state('');
  let duration = $state('01:00');
  let text = $state('');
  let error = $state('');
  let nameInput = $state<HTMLInputElement>();

  $effect(() => {
    path = settings.get<string>(`quick.last_path.${kind}`, '');
  });

  // Folder data for suggestions.
  $effect(() => {
    if ((kind === 'note' || (kind === 'folder' && tree === 'notes')) && !notes.folders.length) notes.load();
    if ((kind === 'task' || (kind === 'folder' && tree === 'tasks')) && !tasks.folders.length) tasks.load();
  });

  const inTasks = $derived(kind === 'task' || (kind === 'folder' && tree === 'tasks'));
  const folders = $derived(inTasks ? tasks.folders : notes.folders);
  const target = $derived(path.trim().replace(/\/+$/, ''));
  const willCreate = $derived(target !== '' && !folders.includes(target));
  const dur = $derived(parseDuration(duration));

  const problem = $derived.by(() => {
    if (kind === 'timer') return dur.ok ? '' : dur.error;
    if (target && !isValidPath(target)) return 'Folder names: a–z, 0–9 and _ only';
    if (!name.trim()) return '';
    if (kind === 'folder') return nameError(name.trim(), folders.includes(join(target, name.trim())) ? [name.trim()] : []);
    if (kind === 'note' && !nameFromTitle(name)) return 'The name needs a letter or digit';
    return '';
  });

  async function create() {
    if (problem || (kind !== 'timer' && !name.trim())) return;
    error = '';
    try {
      if (kind === 'timer') {
        const t = await timers.add(text, dur.ok ? dur.ms : 0);
        if (t) toast.show(`Timer started · ${dur.ok ? dur.label : ''}${text.trim() ? ` · ${text.trim()}` : ''}`);
        onclose();
        return;
      }
      settings.set(settingKey, target);
      if (kind === 'note') {
        const row = await notes.create(target, nameFromTitle(name));
        onclose();
        goto(routes.note(row.id, true));
      } else if (kind === 'task') {
        const t = await tasks.create({ path: target, title: name.trim() });
        onclose();
        goto(routes.task(t.id));
      } else {
        const full = join(target, name.trim());
        if (tree === 'notes') await notes.createFolder(full);
        else await tasks.createFolder(full);
        onclose();
        goto(tree === 'notes' ? routes.notesFolder(full) : routes.tasksFolder(full));
      }
    } catch (e) {
      error = messageOf(e);
    }
  }

  function focusSelect(node: HTMLInputElement) {
    node.focus();
    node.select();
  }
</script>

<Panel label={TITLES[kind]} {onclose}>
  {#snippet head()}
    <span>{TITLES[kind]}</span>
    {#if kind === 'folder'}
      <span class="seg" role="group" aria-label="Folder for">
        <button class:on={tree === 'notes'} onclick={() => (tree = 'notes')}>Notes</button>
        <button class:on={tree === 'tasks'} onclick={() => (tree = 'tasks')}>Tasks</button>
      </span>
    {/if}
    <span class="k">{keyLabel(KEYS[kind])}</span>
  {/snippet}

  {#if kind === 'timer'}
    <div class="row">
      <input
        class="time"
        class:bad={!dur.ok}
        aria-label="Duration, minutes and seconds"
        bind:value={duration}
        onkeydown={(e) => e.key === 'Enter' && create()}
        use:focusSelect />
      <span class="vsep"></span>
      <input class="qi" aria-label="What for" placeholder="what for (optional)" bind:value={text} onkeydown={(e) => e.key === 'Enter' && create()} />
    </div>
  {:else}
    <div class="row">
      <span class="ql">in</span>
      <PathField bind:value={path} {folders} onenter={() => nameInput?.focus()} />
    </div>
    <div class="row">
      <span class="ql">name</span>
      <input
        bind:this={nameInput}
        class="qi"
        aria-label="Name"
        placeholder={kind === 'note' ? 'Note title' : kind === 'task' ? 'Task title' : 'folder_name'}
        bind:value={name}
        oninput={() => (error = '')}
        onkeydown={(e) => e.key === 'Enter' && create()}
        use:focusSelect />
    </div>
  {/if}

  {#snippet foot()}
    {#if error || problem}
      <span class="err">{error || problem}</span>
    {:else if kind === 'timer'}
      <span class="note">mm:ss · minutes any length</span>
    {:else if willCreate}
      <span class="note">will create {target}/</span>
    {/if}
    <span class="r">{kind === 'timer' ? 'tab · ↵ start · esc cancel' : '↵ create · esc cancel'}</span>
  {/snippet}
</Panel>

<style>
  .row {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 42px;
    padding: 0 14px;
    border-bottom: 1px solid var(--c-chip);
  }

  .row:last-of-type {
    border-bottom: 0;
  }

  .ql {
    flex: 0 0 40px;
    font: 400 12px/16px var(--font-ui);
    color: var(--c-faint);
  }

  .qi {
    flex: 1;
    min-width: 0;
    height: 28px;
    border: 0;
    outline: none;
    background: transparent;
    font: 400 15px/22px var(--font-ui);
  }

  .qi::placeholder {
    color: var(--c-faint);
  }

  .time {
    flex: 0 0 72px;
    height: 28px;
    border: 0;
    outline: none;
    background: transparent;
    font: 500 15px/22px var(--font-mono);
  }

  .time.bad {
    color: var(--danger);
  }

  .vsep {
    width: 1px;
    height: 18px;
    background: var(--c-chip);
  }

  .seg {
    display: flex;
    gap: 2px;
  }

  .seg button {
    height: 24px;
    padding: 0 8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--c-muted-2);
    font: 400 12px/16px var(--font-ui);
    cursor: pointer;
  }

  .seg button.on {
    background: var(--c-chip);
    color: var(--text);
  }
</style>
