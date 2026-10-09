<script lang="ts">
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import type { QuickKind } from '#lib/types/ui.type.ts';
  import { settingDefaults, settingKeys } from '#lib/config/settings.config.ts';
  import { timerStartedText } from '#lib/util/timer-text.ts';
  // ⌘K: commands in groups, title search across notes and tasks, and quick-create phrases
  // ("task: …", "note: …", "person: …", "timer 10m …"). ↑↓ move, ↵ runs, Esc closes.
  import type { Command } from '#lib/types/command.type.ts';
  import type { PaletteItem } from '#lib/types/palette.type.ts';
  import type { Phrase } from '#lib/types/quick-parse.type.ts';
  import { goto } from '$app/navigation';
  import Icon from './Icon.svelte';
  import Modal from './Modal.svelte';
  import { available, commands } from '#lib/commands.ts';
  import { peopleAPI } from '#lib/api/people.ts';
  import { notes } from '#lib/stores/notes.svelte.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { timers } from '#lib/stores/timers.svelte.ts';
  import { people } from '#lib/stores/people.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { keyParts } from '#lib/util/keys.ts';
  import { nextIndex, prevIndex } from '#lib/util/list-nav.ts';
  import { groupItems, hasAllWords } from '#lib/util/palette-items.ts';
  import { parsePhrase } from '#lib/util/quick-parse.ts';
  import { routes } from '#lib/util/routes.ts';

  let { onclose }: { onclose: () => void } = $props();

  const ICONS: Record<string, string> = { 'Go to': '→', Create: '+', Notes: '¶', Tasks: '☐', App: '⌘' };
  const MAX_CMDS = 8;
  const MAX_HITS = 4;
  const WIDTH = 600;
  const TOP = 96;

  let query = $state('');
  let hi = $state(0);

  if (!notes.notes.length) notes.load();
  if (!tasks.tasks.length) tasks.load();

  const lastPath = (kind: QuickKind) => settings.get<string>(settingKeys.quickLastPath(kind), '');
  const taskRoot = $derived(settings.get(settingKeys.tasksRootName, settingDefaults.tasksRootName));

  // ---- quick-create phrases

  const createItem = (item: Omit<PaletteItem, 'group' | 'icon' | 'isNew'>): PaletteItem => ({
    ...item,
    group: 'Create',
    icon: '+',
    isNew: true,
  });

  function taskPhrase(title: string): PaletteItem {
    const path = lastPath('task');
    const where = path || taskRoot;
    const status = tasks.statuses[0]?.label ?? 'Not started';
    return createItem({
      key: 'p:task',
      label: `Task: ${title}`,
      sub: `${status} · ${where}`,
      run: async () => {
        await tasks.create({ path, title });
        toast.show(`Task added to ${where}`);
      },
    });
  }

  function notePhrase(title: string): PaletteItem {
    const path = lastPath('note');
    const folder = path ? `${path}/` : '';
    return createItem({
      key: 'p:note',
      label: `Note: ${title}`,
      sub: `notes/${folder}`,
      run: async () => {
        const row = await notes.create({ folder: path, title });
        goto(routes.note(row.id, { edit: true }));
      },
    });
  }

  function personPhrase(first: string, last: string): PaletteItem {
    const fullName = [first, last].filter(Boolean).join(' ');
    return createItem({
      key: 'p:person',
      label: `Person: ${fullName}`,
      sub: 'People',
      run: async () => {
        const id = await peopleAPI.create({ first, last });
        await people.load();
        goto(routes.person(id, { isNew: true }));
      },
    });
  }

  function timerPhrase(p: Extract<Phrase, { kind: 'timer' }>): PaletteItem {
    const label = p.title ? `${p.label}: ${p.title}` : p.label;
    return createItem({
      key: 'p:timer',
      label: `Timer ${label}`,
      sub: 'starts now',
      run: async () => {
        const started = await timers.add(p.title, p.ms);
        if (started) toast.show(timerStartedText({ label: p.label, title: p.title }));
      },
    });
  }

  function phraseItem(p: Phrase): PaletteItem {
    switch (p.kind) {
      case 'task':
        return taskPhrase(p.title);
      case 'note':
        return notePhrase(p.title);
      case 'person':
        return personPhrase(p.first, p.last);
      case 'timer':
        return timerPhrase(p);
    }
  }

  // ---- commands and search hits

  const isTopGroup = (c: Command) => c.group === 'Go to' || c.group === 'Create';

  function commandItem(c: Command, { searching }: { searching: boolean }): PaletteItem {
    const group = isTopGroup(c) ? c.group : 'Commands';
    const sub = searching ? undefined : c.example;
    return { key: c.id, group, icon: ICONS[c.group], label: c.label, sub, keys: c.keys, run: c.run };
  }

  function commandHits(q: string): PaletteItem[] {
    const usable = commands.filter(available);
    if (!q) return usable.filter(isTopGroup).map((c) => commandItem(c, { searching: false }));
    const found = usable.filter((c) => hasAllWords(`${c.label} ${c.group}`, q)).slice(0, MAX_CMDS);
    return found.map((c) => commandItem(c, { searching: true }));
  }

  function noteHits(q: string): PaletteItem[] {
    const found = notes.notes.filter((n) => hasAllWords(`${n.name} ${n.folder}`, q)).slice(0, MAX_HITS);
    return found.map((n) => ({
      key: `n:${n.id}`,
      group: 'Notes',
      icon: ICONS.Notes,
      label: n.name,
      sub: n.folder || undefined,
      run: () => goto(routes.note(n.id)),
    }));
  }

  function taskHits(q: string): PaletteItem[] {
    const found = tasks.tasks.filter((t) => hasAllWords(t.title, q)).slice(0, MAX_HITS);
    return found.map((t) => ({
      key: `t:${t.id}`,
      group: 'Tasks',
      icon: ICONS.Tasks,
      label: t.title,
      sub: t.path || undefined,
      run: () => goto(routes.task(t.id)),
    }));
  }

  const items = $derived.by((): PaletteItem[] => {
    const q = query.trim();
    const phrase = parsePhrase(q);
    const phrases = phrase ? [phraseItem(phrase)] : [];
    if (!q) return [...phrases, ...commandHits(q)];
    return [...phrases, ...commandHits(q), ...noteHits(q), ...taskHits(q)];
  });

  const groups = $derived(groupItems(items));

  async function run(item: PaletteItem | undefined) {
    if (!item) return;
    onclose();
    await item.run();
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      hi = nextIndex(hi, items.length);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      hi = prevIndex(hi);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      run(items[hi]);
    }
  }

  function scrollIntoView(node: HTMLElement, active: boolean) {
    const reveal = (on: boolean) => {
      if (on) node.scrollIntoView({ block: 'nearest' });
    };
    reveal(active);
    return { update: reveal };
  }
</script>

<Modal label="Command palette" width={WIDTH} top={TOP} shade {onclose}>
  <div class="pal">
    <div class="pin">
      <span class="ic"><Icon name="search" size={16} /></span>
      <!-- svelte-ignore a11y_autofocus -->
      <input use:plainText aria-label="Command or search" placeholder="Type a command, or search notes and tasks…" autofocus bind:value={query} oninput={() => (hi = 0)} {onkeydown} />
      <kbd>esc</kbd>
    </div>
    <div class="res" role="listbox">
      {#each groups as g (g.name)}
        <div class="grp">{g.name}</div>
        {#each g.items as { item, index } (item.key)}
          <button
            class="it"
            class:on={index === hi}
            role="option"
            aria-selected={index === hi}
            onmouseenter={() => (hi = index)}
            onclick={() => run(item)}
            use:scrollIntoView={index === hi}>
            <span class="gl">{item.icon}</span>
            <span class="lb"><span class:new={item.isNew}>{item.label}</span>{#if item.sub}<span class="sub">&nbsp;· {item.sub}</span>{/if}</span>
            {#if item.keys}<span class="keys">{#each keyParts(item.keys) as k, i (i)}<kbd>{k}</kbd>{/each}</span>{/if}
          </button>
        {/each}
      {/each}
      {#if items.length === 0}<div class="empty">Nothing matches. Try “task: …” or “timer 10m …”.</div>{/if}
    </div>
    <div class="foot">
      <span><kbd>↑</kbd><kbd>↓</kbd> move</span>
      <span><kbd>↵</kbd> run</span>
      <span><kbd>esc</kbd> close</span>
      <span class="r">All commands in Settings → Command palette</span>
    </div>
  </div>
</Modal>

<style>
  .pal {
    display: flex;
    flex-direction: column;
    border-radius: 10px;
    overflow: hidden;
  }

  .pin {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 52px;
    padding: 0 16px;
    border-bottom: 1px solid var(--border);
  }

  .ic {
    display: grid;
    color: var(--c-muted-2);
  }

  .pin input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: none;
    background: transparent;
    font: 400 16px/22px var(--font-ui);
  }

  .pin input::placeholder {
    color: var(--c-muted-2);
  }

  .res {
    max-height: 420px;
    overflow: auto;
    padding: 6px 0;
  }

  .grp {
    padding: 8px 16px 4px;
    font: 700 11px/16px var(--font-ui);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--c-muted-2);
  }

  .it {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    min-height: 36px;
    padding: 0 16px;
    border: 0;
    background: transparent;
    text-align: left;
    font: var(--text-ui);
    color: var(--text);
    cursor: pointer;
  }

  .it.on {
    background: var(--c-selected);
  }

  .gl {
    flex: 0 0 16px;
    display: grid;
    place-items: center;
    color: var(--c-muted-2);
  }

  .lb {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sub {
    color: var(--c-muted-2);
    font-size: 13px;
  }

  .new {
    color: var(--primary);
  }

  .keys {
    display: flex;
    gap: 4px;
  }

  .empty {
    padding: 20px 16px;
    font: var(--text-caption);
    color: var(--c-muted-2);
  }

  .foot {
    display: flex;
    align-items: center;
    gap: 14px;
    height: 34px;
    padding: 0 16px;
    border-top: 1px solid var(--border);
    font: var(--text-meta);
    color: var(--c-muted-2);
  }

  .foot span {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .foot .r {
    margin-left: auto;
  }
</style>
