<script lang="ts">
  // Dev only: every shared component with sample data, for checking states against the mockups.
  import type { DayRange } from '#lib/types/date.type.ts';
  import type { Leaf, ExplorerItem } from '#lib/types/explorer.type.ts';
  import Explorer from '#lib/components/Explorer.svelte';
  import Section from '#lib/components/Section.svelte';
  import MarkdownField from '#lib/components/MarkdownField.svelte';
  import EditControls from '#lib/components/EditControls.svelte';
  import TagEditor from '#lib/components/TagEditor.svelte';
  import DateField from '#lib/components/DateField.svelte';
  import TimeField from '#lib/components/TimeField.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import { tagPool } from '#lib/stores/tags.svelte.ts';
  import { baseOf, isUnder, join, parentOf } from '#lib/util/names.ts';
  import { today } from '#lib/util/dates.ts';
  import { nav } from '#lib/stores/history.svelte.ts';
  import { routes } from '#lib/util/routes.ts';

  if (!import.meta.env.DEV) nav.replace(routes.notes);
  tagPool.names = ['ci', 'docs', 'home', 'reading', 'release', 'tauri', 'ui', 'v0.1'];

  let folders = $state(['home', 'home/recipes', 'inbox', 'work', 'work/meetings', 'work/tend_v0_1']);
  let leaves = $state<Leaf[]>([
    { id: 'n1', folder: '', name: 'reading_list' },
    { id: 'n2', folder: 'work', name: 'weekly_plan' },
    { id: 'n3', folder: 'work/meetings', name: 'standup_0310' },
    { id: 'n5', folder: 'work/tend_v0_1', name: 'ideas' },
  ]);
  let selected = $state<ExplorerItem | null>({ kind: 'folder', path: 'work' });

  /** `path` with its `from` prefix swapped for `to`, when it sits under `from`. */
  function rebase(path: string, { from, to }: { from: string; to: string }) {
    if (!from || !isUnder(path, from)) return path;
    return to + path.slice(from.length);
  }

  const actions = {
    open: (item: ExplorerItem) => {
      selected = item;
    },
    createFolder: async ({ parent, name }: { parent: string; name: string }) => {
      const path = join(parent, name);
      folders = [...folders, path];
      return path;
    },
    createLeaf: async ({ folder, name }: { folder: string; name: string }) => {
      const id = `n${Date.now()}`;
      leaves = [...leaves, { id, folder, name }];
      selected = { kind: 'leaf', id };
    },
    rename: async (item: ExplorerItem, name: string) => {
      if (item.kind === 'leaf') leaves = leaves.map((l) => (l.id === item.id ? { ...l, name } : l));
      else move({ from: item.path, to: join(parentOf(item.path), name) });
    },
    move: async (item: ExplorerItem, target: string) => {
      if (item.kind === 'leaf') leaves = leaves.map((l) => (l.id === item.id ? { ...l, folder: target } : l));
      else move({ from: item.path, to: join(target, baseOf(item.path)) });
    },
    remove: async (item: ExplorerItem) => {
      if (item.kind === 'leaf') {
        leaves = leaves.filter((l) => l.id !== item.id);
        return;
      }
      folders = folders.filter((f) => !isUnder(f, item.path));
      leaves = leaves.filter((l) => !isUnder(l.folder, item.path));
    },
    reload: async () => {},
    contents: (path: string) => {
      const n = leaves.filter((l) => isUnder(l.folder, path)).length;
      const f = folders.filter((x) => x !== path && isUnder(x, path)).length;
      return `It contains ${n} notes and ${f} folders. Everything inside is deleted from disk.`;
    },
  };

  function move(paths: { from: string; to: string }) {
    folders = folders.map((f) => rebase(f, paths));
    leaves = leaves.map((l) => ({ ...l, folder: rebase(l.folder, paths) }));
  }

  let desc = $state('First signed build for **macOS** and Linux.\n\n- [x] sign\n- [ ] notarize');
  let field = $state<MarkdownField>();
  let editing = $state(false);
  let tags = $state(['v0.1', 'tauri']);
  let due = $state<string | null>(today());
  let time = $state<string | null>('17:30');
  let range = $state<DayRange>([today(), today()]);
  let popAnchor = $state<HTMLButtonElement>();
  let popOpen = $state(false);
</script>

<Explorer id="playground" rootName="notes" {folders} {leaves} {selected} {actions} />

<main class="main">
  <h1 class="heading">Playground</h1>
  <p class="caption muted">Selected: {JSON.stringify(selected)}</p>

  <div class="props">
    <span class="pl">Tags</span>
    <TagEditor {tags} onchange={(t) => (tags = t)} addLabel="+" />
    <span class="pl">Due</span>
    <span class="pv"><DateField value={due} onchange={(d) => (due = d)} /><TimeField value={time} onchange={(t) => (time = t)} /></span>
    <span class="pl">DayRange</span>
    <span class="pv"><DateField mode="range" {range} onrange={(r) => (range = r)} active /> <span class="caption muted">{range.join(' → ')} · {time ?? 'no time'}</span></span>
    <span class="pl">Popover</span>
    <span class="pv">
      <button class="btn-ghost" bind:this={popAnchor} onclick={() => (popOpen = !popOpen)}>Open popover</button>
      {#if popOpen}
        <Popover anchor={popAnchor} pad role="listbox" onclose={() => (popOpen = false)}>
          {#each ['Not started', 'In progress', 'Done'] as s (s)}<button class="opt" onclick={() => (popOpen = false)}>{s}</button>{/each}
        </Popover>
      {/if}
    </span>
  </div>

  <Section title="Description">
    {#snippet actions()}
      <EditControls {editing} what="description" onedit={() => field?.edit()} oncancel={() => field?.cancel()} onsave={() => field?.save()} />
    {/snippet}
    <MarkdownField bind:this={field} bind:editing value={desc} onsave={(t) => { desc = t; }} placeholder="Add a description…" empty="No description" label="Description" />
  </Section>
</main>

<style>
  .main {
    flex: 1;
    padding: 40px 24px;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 24px;
    max-width: 760px;
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
  }
</style>
