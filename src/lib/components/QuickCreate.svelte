<script lang="ts">
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { settingKeys } from '#lib/config/settings.config.ts';
  // Quick create: ⌘N note · ⌘⇧T task · ⌘⇧N folder (Notes/Tasks) · ⌘T timer.
  import type { QuickKind } from '#lib/types/ui.type.ts';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import Panel from './Panel.svelte';
  import PathField from './PathField.svelte';
  import { focusAndSelect } from '#lib/actions/focus.action.ts';
  import { messageOf } from '#lib/errors/app-error.ts';
  import { notes } from '#lib/stores/notes.svelte.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { timers } from '#lib/stores/timers.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { cleanPath, isValidPath, join, nameError } from '#lib/util/names.ts';
  import { parseDuration } from '#lib/util/timer-parse.ts';
  import { timerStartedText } from '#lib/util/timer-text.ts';
  import { label as keyLabel } from '#lib/util/keys.ts';
  import { routes } from '#lib/util/routes.ts';

  let { kind, onclose }: { kind: QuickKind; onclose: () => void } = $props();

  const TITLES: Record<QuickKind, string> = { note: 'New note', task: 'New task', folder: 'New folder', timer: 'New timer' };
  const KEYS: Record<QuickKind, string> = { note: 'mod+n', task: 'mod+shift+t', folder: 'mod+shift+n', timer: 'mod+t' };
  const PLACEHOLDERS: Record<Exclude<QuickKind, 'timer'>, string> = { note: 'Note title', task: 'Task title', folder: 'folder_name' };

  // Folder: Notes or Tasks, defaulting to the section you're in.
  let tree = $state<'notes' | 'tasks'>(page.url.pathname.startsWith(routes.tasks) ? 'tasks' : 'notes');
  const settingKey = $derived(settingKeys.quickLastPath(kind));
  let path = $state('');
  let name = $state('');
  let duration = $state('01:00');
  let text = $state('');
  let error = $state('');
  let nameInput = $state<HTMLInputElement>();

  $effect(() => {
    path = settings.get<string>(settingKey, '');
  });

  const inTasks = $derived(kind === 'task' || (kind === 'folder' && tree === 'tasks'));
  const inNotes = $derived(kind !== 'timer' && !inTasks);
  const folders = $derived(inTasks ? tasks.folders : notes.folders);
  const target = $derived(cleanPath(path));
  const title = $derived(name.trim());
  const willCreate = $derived(target !== '' && !folders.includes(target));
  const dur = $derived(parseDuration(duration));

  // Folder data for suggestions.
  $effect(() => {
    if (inNotes && !notes.folders.length) notes.load();
    if (inTasks && !tasks.folders.length) tasks.load();
  });

  const problem = $derived.by(() => {
    if (kind === 'timer') return dur.ok ? '' : dur.error;
    if (target && !isValidPath(target)) return 'Folder names: a–z and 0–9, with single _ between them';
    if (!title) return '';
    if (kind === 'folder') return folderProblem();
    return '';
  });
  const message = $derived(error || problem);

  function folderProblem() {
    const taken = folders.includes(join(target, title));
    return nameError(title, taken ? [title] : []);
  }

  async function create() {
    if (problem) return;
    if (kind === 'timer') return createTimer();
    if (!title) return;
    error = '';
    try {
      settings.set(settingKey, target);
      if (kind === 'note') await createNote();
      else if (kind === 'task') await createTask();
      else await createFolder();
    } catch (e) {
      error = messageOf(e);
    }
  }

  async function createTimer() {
    if (!dur.ok) return;
    error = '';
    const whatFor = text.trim();
    const t = await timers.add(text, dur.ms);
    if (t) toast.show(timerStartedText({ label: dur.label, title: whatFor }));
    onclose();
  }

  async function createNote() {
    const row = await notes.create({ folder: target, title });
    onclose();
    goto(routes.note(row.id, { edit: true }));
  }

  async function createTask() {
    const t = await tasks.create({ path: target, title });
    onclose();
    goto(routes.task(t.id));
  }

  async function createFolder() {
    const full = join(target, title);
    if (tree === 'notes') await notes.createFolder(full);
    else await tasks.createFolder(full);
    onclose();
    goto(tree === 'notes' ? routes.notesFolder(full) : routes.tasksFolder(full));
  }

  function onEnter(e: KeyboardEvent) {
    if (e.key === 'Enter') create();
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
      <input use:plainText
        class="time"
        class:bad={!dur.ok}
        aria-label="Duration, minutes and seconds"
        bind:value={duration}
        onkeydown={onEnter}
        use:focusAndSelect />
      <span class="vsep"></span>
      <input use:plainText class="qi" aria-label="What for" placeholder="what for (optional)" bind:value={text} onkeydown={onEnter} />
    </div>
  {:else}
    <div class="row">
      <span class="ql">in</span>
      <PathField bind:value={path} {folders} onenter={() => nameInput?.focus()} />
    </div>
    <div class="row">
      <span class="ql">name</span>
      <input use:plainText
        bind:this={nameInput}
        class="qi"
        aria-label="Name"
        placeholder={PLACEHOLDERS[kind]}
        bind:value={name}
        oninput={() => (error = '')}
        onkeydown={onEnter}
        use:focusAndSelect />
    </div>
  {/if}

  {#snippet foot()}
    {#if message}
      <span class="err">{message}</span>
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
