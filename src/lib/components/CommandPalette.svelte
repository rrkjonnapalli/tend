<script lang="ts">
  // ⌘K: commands in groups, title search across notes and tasks, and quick-create phrases
  // ("task: …", "note: …", "person: …", "timer 10m …"). ↑↓ move, ↵ runs, Esc closes.
  import type { Command } from '#lib/types/command.type.ts';
  import type { Phrase } from '#lib/types/quick-parse.type.ts';
  import { goto } from '$app/navigation';
  import Icon from './Icon.svelte';
  import Modal from './Modal.svelte';
  import { available, commands } from '#lib/commands.ts';
  import { peopleAPI } from '#lib/api/people.ts';
  import { messageOf } from '#lib/api/core.ts';
  import { notes } from '#lib/stores/notes.svelte.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { timers } from '#lib/stores/timers.svelte.ts';
  import { people } from '#lib/stores/people.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { keyParts } from '#lib/util/keys.ts';
  import { nameFromTitle, parsePhrase } from '#lib/util/quick-parse.ts';
  import { routes } from '#lib/util/routes.ts';

  let { onclose }: { onclose: () => void } = $props();

  type Item = { key: string; group: string; icon: string; label: string; sub?: string; keys?: string; isNew?: boolean; run: () => void | Promise<void> };

  const ICONS: Record<string, string> = { 'Go to': '→', Create: '+', Notes: '¶', Tasks: '☐', App: '⌘' };
  const MAX_CMDS = 8;
  const MAX_HITS = 4;

  let query = $state('');
  let hi = $state(0);

  if (!notes.notes.length) notes.load();
  if (!tasks.tasks.length) tasks.load();

  const lastPath = (kind: string) => settings.get<string>(`quick.last_path.${kind}`, '');
  const taskRoot = $derived(settings.get('tasks.root_name', 'tasks'));

  function phraseItem(p: Phrase): Item {
    switch (p.kind) {
      case 'task': {
        const path = lastPath('task');
        return {
          key: 'p:task', group: 'Create', icon: '+', isNew: true,
          label: `Task: ${p.title}`, sub: `${tasks.statuses[0]?.label ?? 'Not started'} · ${path || taskRoot}`,
          run: async () => {
            await tasks.create({ path, title: p.title });
            toast.show(`Task added to ${path || taskRoot}`);
          },
        };
      }
      case 'note': {
        const path = lastPath('note');
        return {
          key: 'p:note', group: 'Create', icon: '+', isNew: true,
          label: `Note: ${p.title}`, sub: `notes/${path ? `${path}/` : ''}${nameFromTitle(p.title)}.md`,
          run: async () => {
            const row = await notes.create(path, nameFromTitle(p.title));
            goto(routes.note(row.id, true));
          },
        };
      }
      case 'person':
        return {
          key: 'p:person', group: 'Create', icon: '+', isNew: true,
          label: `Person: ${[p.first, p.last].filter(Boolean).join(' ')}`, sub: 'People',
          run: async () => {
            const id = await peopleAPI.create({ first: p.first, last: p.last });
            await people.load();
            goto(routes.person(id));
          },
        };
      case 'timer':
        return {
          key: 'p:timer', group: 'Create', icon: '+', isNew: true,
          label: `Timer ${p.label}${p.title ? `: ${p.title}` : ''}`, sub: 'starts now',
          run: async () => {
            if (await timers.add(p.title, p.ms)) toast.show(`Timer started · ${p.label}${p.title ? ` · ${p.title}` : ''}`);
          },
        };
    }
  }

  const words = (q: string) => q.toLowerCase().split(/\s+/).filter(Boolean);
  const hit = (text: string, q: string) => words(q).every((w) => text.toLowerCase().includes(w));

  function commandItem(c: Command, searching: boolean): Item {
    const group = c.group === 'Go to' || c.group === 'Create' ? c.group : 'Commands';
    return { key: c.id, group, icon: ICONS[c.group], label: c.label, sub: searching ? undefined : c.example, keys: c.keys, run: c.run };
  }

  const items = $derived.by((): Item[] => {
    const q = query.trim();
    const out: Item[] = [];
    const phrase = parsePhrase(q);
    if (phrase) out.push(phraseItem(phrase));
    const usable = commands.filter(available);
    if (!q) {
      usable.filter((c) => c.group === 'Go to' || c.group === 'Create').forEach((c) => out.push(commandItem(c, false)));
      return out;
    }
    usable
      .filter((c) => hit(`${c.label} ${c.group}`, q))
      .slice(0, MAX_CMDS)
      .forEach((c) => out.push(commandItem(c, true)));
    notes.notes
      .filter((n) => hit(`${n.title} ${n.name}`, q))
      .slice(0, MAX_HITS)
      .forEach((n) => out.push({ key: `n:${n.id}`, group: 'Notes', icon: '¶', label: n.title, sub: n.folder || undefined, run: () => goto(routes.note(n.id)) }));
    tasks.tasks
      .filter((t) => hit(t.title, q))
      .slice(0, MAX_HITS)
      .forEach((t) => out.push({ key: `t:${t.id}`, group: 'Tasks', icon: '☐', label: t.title, sub: t.path || undefined, run: () => goto(routes.task(t.id)) }));
    return out;
  });

  // Same group order as they first appear.
  const groups = $derived.by(() => {
    const out: { name: string; items: { item: Item; index: number }[] }[] = [];
    items.forEach((item, index) => {
      let g = out.find((x) => x.name === item.group);
      if (!g) out.push((g = { name: item.group, items: [] }));
      g.items.push({ item, index });
    });
    return out;
  });

  async function run(item: Item | undefined) {
    if (!item) return;
    onclose();
    try {
      await item.run();
    } catch (e) {
      toast.show(messageOf(e));
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      hi = Math.min(hi + 1, items.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      hi = Math.max(hi - 1, 0);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      run(items[hi]);
    }
  }

  function scrollIntoView(node: HTMLElement, on: boolean) {
    const go = (v: boolean) => v && node.scrollIntoView({ block: 'nearest' });
    go(on);
    return { update: go };
  }
</script>

<Modal label="Command palette" width={600} top={96} shade {onclose}>
  <div class="pal">
    <div class="pin">
      <span class="ic"><Icon name="search" size={16} /></span>
      <!-- svelte-ignore a11y_autofocus -->
      <input aria-label="Command or search" placeholder="Type a command, or search notes and tasks…" autofocus bind:value={query} oninput={() => (hi = 0)} {onkeydown} />
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
    font: 400 14px/20px var(--font-ui);
    color: var(--text);
    cursor: pointer;
  }

  .it.on {
    background: var(--c-chip);
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
    font: 400 13px/18px var(--font-ui);
    color: var(--c-muted-2);
  }

  .foot {
    display: flex;
    align-items: center;
    gap: 14px;
    height: 34px;
    padding: 0 16px;
    border-top: 1px solid var(--border);
    font: 400 12px/16px var(--font-ui);
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
