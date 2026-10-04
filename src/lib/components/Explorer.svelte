<script lang="ts" module>
  export type Leaf = { id: string; folder: string; name: string };
  export type ExplorerItem = { kind: 'folder'; path: string } | { kind: 'leaf'; id: string };

  export type ExplorerActions = {
    open: (item: ExplorerItem) => void;
    createFolder: (parent: string, name: string) => Promise<void>;
    /** Notes: create a leaf inline. Left out for explorers that hold folders only. */
    createLeaf?: (folder: string, name: string) => Promise<void>;
    /** The "new item" toolbar button when there is no inline leaf creation (tasks). */
    newItem?: (folder: string) => void;
    rename: (item: ExplorerItem, name: string) => Promise<void>;
    move: (item: ExplorerItem, target: string) => Promise<void>;
    remove: (item: ExplorerItem) => Promise<void>;
    reload: () => Promise<void>;
    /** "5 notes and 2 folders" for the delete dialog. */
    contents: (path: string) => string;
  };

  // Expanded folders per explorer, kept while the app runs so switching sections keeps them.
  const expandedBy = new Map<string, Record<string, boolean>>();
</script>

<script lang="ts">
  // The left-hand tree shared by Notes (folders + notes) and Tasks (folders only).
  import { untrack } from 'svelte';
  import Icon from './Icon.svelte';
  import Popover from './Popover.svelte';
  import MoveToDialog from './MoveToDialog.svelte';
  import DeleteDialog from './DeleteDialog.svelte';
  import { baseOf, isUnder, join, nameError, parentOf } from '#lib/util/names.ts';
  import { label as keyLabel, matches } from '#lib/util/keys.ts';
  import { hasLayers } from '#lib/util/layers.ts';
  import { messageOf } from '#lib/api/core.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';

  let {
    id,
    rootName,
    folders,
    leaves = [],
    selected,
    newItemLabel = 'New note',
    actions,
  }: {
    /** Remembers expanded folders per explorer ('notes', 'tasks'). */
    id: string;
    rootName: string;
    /** Every folder path except the root (''). */
    folders: string[];
    leaves?: Leaf[];
    selected: ExplorerItem | null;
    newItemLabel?: string;
    actions: ExplorerActions;
  } = $props();

  type Row = {
    key: string;
    item: ExplorerItem;
    name: string;
    depth: number;
    folder: string;
    isFolder: boolean;
    isNew?: 'folder' | 'leaf';
  };

  // The explorer id never changes for a mounted explorer.
  const memoryKey = untrack(() => id);
  let expanded = $state<Record<string, boolean>>(expandedBy.get(memoryKey) ?? { '': true });
  $effect(() => {
    expandedBy.set(memoryKey, expanded);
  });

  // Editing: an existing row being renamed, or a new row being named.
  let editing = $state<{ key: string; folder: string; isNew?: 'folder' | 'leaf'; item?: ExplorerItem } | null>(null);
  let draft = $state('');
  let menu = $state<{ row: Row; anchor: HTMLElement | { x: number; y: number } } | null>(null);
  let moving = $state<Row | null>(null);
  let deleting = $state<Row | null>(null);
  let drag = $state<{ row: Row; over: string | null; x: number; y: number } | null>(null);
  let suppressClick = false;

  const keyOf = (item: ExplorerItem) => (item.kind === 'folder' ? `f:${item.path}` : `l:${item.id}`);
  const selectedKey = $derived(selected ? keyOf(selected) : null);
  const selectedFolder = $derived.by(() => {
    if (!selected) return '';
    if (selected.kind === 'folder') return selected.path;
    return leaves.find((l) => l.id === selected.id)?.folder ?? '';
  });

  // Opening an item reveals it: every folder above it expands.
  $effect(() => {
    let p = selected?.kind === 'folder' ? parentOf(selected.path) : selectedFolder;
    const next = { ...expanded };
    let changed = false;
    while (true) {
      if (!next[p]) {
        next[p] = true;
        changed = true;
      }
      if (p === '') break;
      p = parentOf(p);
    }
    if (changed) expanded = next;
  });

  const childrenOf = (p: string) => folders.filter((f) => parentOf(f) === p).sort();
  const leavesIn = (p: string) => leaves.filter((l) => l.folder === p).sort((a, b) => a.name.localeCompare(b.name));

  const rows = $derived.by(() => {
    const out: Row[] = [];
    const walk = (path: string, depth: number) => {
      out.push({
        key: `f:${path}`,
        item: { kind: 'folder', path },
        name: path === '' ? rootName : baseOf(path),
        depth,
        folder: parentOf(path),
        isFolder: true,
      });
      if (!expanded[path]) return;
      if (editing?.isNew === 'folder' && editing.folder === path)
        out.push({ key: 'new', item: { kind: 'folder', path: '' }, name: '', depth: depth + 1, folder: path, isFolder: true, isNew: 'folder' });
      childrenOf(path).forEach((c) => walk(c, depth + 1));
      if (editing?.isNew === 'leaf' && editing.folder === path)
        out.push({ key: 'new', item: { kind: 'leaf', id: '' }, name: '', depth: depth + 1, folder: path, isFolder: false, isNew: 'leaf' });
      for (const l of leavesIn(path))
        out.push({ key: `l:${l.id}`, item: { kind: 'leaf', id: l.id }, name: l.name, depth: depth + 1, folder: path, isFolder: false });
    };
    walk('', 0);
    return out;
  });

  const siblingNames = (folder: string, exceptKey?: string) => [
    ...folders.filter((f) => parentOf(f) === folder && `f:${f}` !== exceptKey).map(baseOf),
    ...leaves.filter((l) => l.folder === folder && `l:${l.id}` !== exceptKey).map((l) => l.name),
  ];

  const draftError = $derived(editing ? nameError(draft.trim(), siblingNames(editing.folder, editing.key)) : '');

  function focusEditor(node: HTMLInputElement) {
    node.focus();
    node.select();
  }

  function startRename(row: Row) {
    menu = null;
    editing = { key: row.key, folder: row.folder, item: row.item };
    draft = row.name;
  }

  function startNew(kind: 'folder' | 'leaf') {
    const folder = selectedFolder;
    expanded = { ...expanded, [folder]: true };
    editing = { key: 'new', folder, isNew: kind };
    draft = '';
  }

  async function commit() {
    const e = editing;
    if (!e) return;
    const name = draft.trim();
    if (draftError) {
      if (e.isNew && !name) editing = null;
      return;
    }
    editing = null;
    try {
      if (e.isNew === 'folder') {
        await actions.createFolder(e.folder, name);
        toast.show(`Folder ${join(e.folder, name)} created`);
      } else if (e.isNew === 'leaf') {
        await actions.createLeaf?.(e.folder, name);
      } else if (e.item) {
        const old = e.item.kind === 'folder' ? baseOf(e.item.path) : leaves.find((l) => l.id === (e.item as { id: string }).id)?.name;
        if (old === name) return;
        await actions.rename(e.item, name);
        toast.show(`Renamed to ${name}`);
      }
    } catch (err) {
      toast.show(messageOf(err));
    }
  }

  function onEditKey(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      commit();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      editing = null;
    }
  }

  function onEditBlur() {
    if (!editing) return;
    if (draftError) editing = null;
    else commit();
  }

  // ---- moving

  const clash = (folder: string, name: string) => siblingNames(folder).includes(name);

  function canDrop(row: Row, target: string): boolean {
    if (row.item.kind === 'folder') {
      const path = row.item.path;
      return !isUnder(target, path) && parentOf(path) !== target && !clash(target, baseOf(path));
    }
    return row.folder !== target && !clash(target, row.name);
  }

  async function moveTo(row: Row, target: string) {
    moving = null;
    try {
      await actions.move(row.item, target);
      expanded = { ...expanded, [target]: true };
      toast.show(`Moved to ${target || rootName}/`);
    } catch (err) {
      toast.show(messageOf(err));
    }
  }

  function onRowDown(row: Row, e: PointerEvent) {
    if (e.button !== 0 || row.isNew || editing?.key === row.key || (row.isFolder && row.item.kind === 'folder' && row.item.path === '')) return;
    const x0 = e.clientX;
    const y0 = e.clientY;
    let frame = 0;
    let last: PointerEvent | null = null;

    const step = () => {
      frame = 0;
      if (!last || !drag) return;
      const hit = document.elementFromPoint(last.clientX, last.clientY)?.closest<HTMLElement>('[data-drop]');
      drag = { ...drag, over: hit ? hit.dataset.drop! : null, x: last.clientX + 12, y: last.clientY + 8 };
    };
    const move = (ev: PointerEvent) => {
      if (!drag) {
        if (Math.abs(ev.clientX - x0) + Math.abs(ev.clientY - y0) < 5) return;
        menu = null;
        drag = { row, over: null, x: ev.clientX + 12, y: ev.clientY + 8 };
      }
      last = ev;
      if (!frame) frame = requestAnimationFrame(step);
    };
    const up = () => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
      if (frame) cancelAnimationFrame(frame);
      const d = drag;
      drag = null;
      if (!d) return;
      suppressClick = true;
      setTimeout(() => (suppressClick = false), 0);
      if (d.over !== null && canDrop(d.row, d.over)) moveTo(d.row, d.over);
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  }

  // ---- deleting

  async function confirmDelete(row: Row) {
    deleting = null;
    try {
      await actions.remove(row.item);
      toast.show(`Deleted ${row.isFolder ? `${(row.item as { path: string }).path}/` : row.name}`);
    } catch (err) {
      toast.show(messageOf(err));
    }
  }

  // ---- interactions

  function openRow(row: Row) {
    if (suppressClick || row.isNew || editing?.key === row.key) return;
    menu = null;
    actions.open(row.item);
  }

  function toggle(path: string, e: Event) {
    e.stopPropagation();
    expanded = { ...expanded, [path]: !expanded[path] };
  }

  function openMenu(row: Row, e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    if (row.item.kind === 'folder' && row.item.path === '') return;
    const anchor = e.type === 'contextmenu' ? { x: e.clientX, y: e.clientY } : (e.currentTarget as HTMLElement);
    menu = { row, anchor };
  }

  const selectedRow = $derived(rows.find((r) => r.key === selectedKey && !(r.item.kind === 'folder' && r.item.path === '')));

  function isTyping(t: EventTarget | null) {
    return t instanceof HTMLElement && (t.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName) || !!t.closest('.cm-editor'));
  }

  function onkeydown(e: KeyboardEvent) {
    if (!ui.explorerOpen || hasLayers() || editing || isTyping(e.target) || !selectedRow) return;
    if (e.key === 'F2') {
      e.preventDefault();
      startRename(selectedRow);
    } else if (matches(e, 'mod+shift+m')) {
      e.preventDefault();
      moving = selectedRow;
    } else if (e.key === 'Backspace' && !e.metaKey && !e.ctrlKey) {
      e.preventDefault();
      deleting = selectedRow;
    }
  }

  const indent = (row: Row) => 16 + row.depth * 16 + (row.isFolder ? 0 : 16);
  const dropPath = (row: Row) => (row.isNew ? undefined : row.item.kind === 'folder' ? row.item.path : row.folder);
  const allFolders = $derived(['', ...folders]);
</script>

<svelte:window {onkeydown} />

{#if ui.explorerOpen}
  <aside class="explorer" class:dragging={drag !== null} aria-label="Explorer">
    <div class="toolbar" role="toolbar" aria-label="Explorer actions">
      <button
        class="icon-btn sm"
        aria-label={newItemLabel}
        title={newItemLabel}
        onclick={() => (actions.newItem ? actions.newItem(selectedFolder) : startNew('leaf'))}>
        <Icon name={actions.newItem ? 'taskNew' : 'noteNew'} size={13} stroke={1.75} />
      </button>
      <button class="icon-btn sm" aria-label="New folder" title="New folder" onclick={() => startNew('folder')}>
        <Icon name="folderNew" size={13} stroke={1.75} />
      </button>
      <button class="icon-btn sm" aria-label="Reload" title="Reload" onclick={() => actions.reload()}>
        <Icon name="reload" size={13} stroke={1.75} />
      </button>
      <button class="icon-btn sm" aria-label="Collapse all" title="Collapse all" onclick={() => (expanded = { '': true })}>
        <Icon name="collapseAll" size={13} stroke={1.75} />
      </button>
    </div>

    <div class="tree" role="tree">
      {#each rows as row (row.key)}
        {@const isSel = row.key === selectedKey && !drag}
        {@const isTarget = drag !== null && row.isFolder && !row.isNew && drag.over === (row.item as { path: string }).path}
        {@const isEditing = editing?.key === row.key}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          class="row"
          class:leaf={!row.isFolder}
          class:sel={isSel}
          class:drop={isTarget && canDrop(drag!.row, drag!.over!)}
          class:bad={isTarget && !canDrop(drag!.row, drag!.over!)}
          class:ghosted={drag?.row.key === row.key}
          style:padding-left="{indent(row)}px"
          data-drop={dropPath(row)}
          role="treeitem"
          tabindex="-1"
          aria-selected={isSel}
          aria-expanded={row.isFolder ? !!expanded[(row.item as { path: string }).path] : undefined}
          onpointerdown={(e) => onRowDown(row, e)}
          oncontextmenu={(e) => openMenu(row, e)}
          onclick={() => !row.isFolder && openRow(row)}>
          {#if row.isFolder}
            <button class="chev" aria-label="Expand or collapse folder" onclick={(e) => !row.isNew && toggle((row.item as { path: string }).path, e)}>
              <Icon name={expanded[(row.item as { path: string }).path] && !row.isNew ? 'chevronDown' : 'chevronRight'} size={10} stroke={3} />
            </button>
          {/if}
          {#if isEditing}
            <input
              class="ein"
              class:bad={!!draftError && !!draft.trim()}
              aria-label="Name"
              bind:value={draft}
              onkeydown={onEditKey}
              onblur={onEditBlur}
              onpointerdown={(e) => e.stopPropagation()}
              use:focusEditor />
          {:else if row.isFolder}
            <button class="nm" onclick={() => openRow(row)}>{row.name}</button>
          {:else}
            <span class="nm">{row.name}</span>
          {/if}
          {#if !row.isNew && !isEditing && !(row.isFolder && (row.item as { path: string }).path === '')}
            <button
              class="more"
              class:on={menu?.row.key === row.key}
              aria-label="More actions"
              title="More"
              onpointerdown={(e) => e.stopPropagation()}
              onclick={(e) => openMenu(row, e)}>
              <Icon name="more" size={14} />
            </button>
          {/if}
        </div>
        {#if isEditing && draftError && draft.trim()}
          <div class="name-err" style:padding-left="{indent(row)}px">{draftError}</div>
        {/if}
      {/each}
    </div>

    <button class="caret" aria-label="Collapse explorer" title="Hide explorer" onclick={ui.toggleExplorer}>
      <Icon name="chevronLeft" size={10} stroke={3} />
    </button>
  </aside>
{:else}
  <button class="caret tab" aria-label="Expand explorer" title="Show explorer" onclick={ui.toggleExplorer}>
    <Icon name="chevronRight" size={10} stroke={3} />
  </button>
{/if}

{#if menu}
  {@const row = menu.row}
  <Popover anchor={menu.anchor} placement="bottom-end" role="menu" pad onclose={() => (menu = null)}>
    <div class="menu">
      <button class="opt" role="menuitem" onclick={() => startRename(row)}>Rename<span class="hint">F2</span></button>
      <button class="opt" role="menuitem" onclick={() => ((moving = row), (menu = null))}>
        Move to…<span class="hint">{keyLabel('mod+shift+m')}</span>
      </button>
      <div class="opt-sep"></div>
      <button class="opt danger" role="menuitem" onclick={() => ((deleting = row), (menu = null))}>Delete…<span class="hint">⌫</span></button>
    </div>
  </Popover>
{/if}

{#if moving}
  {@const row = moving}
  <MoveToDialog
    what={row.item.kind === 'folder' ? `${row.item.path}/` : join(row.folder, row.name)}
    folders={allFolders}
    current={row.folder}
    self={row.item.kind === 'folder' ? row.item.path : undefined}
    {rootName}
    onmove={(t) => moveTo(row, t)}
    onclose={() => (moving = null)} />
{/if}

{#if deleting}
  {@const row = deleting}
  {#if row.item.kind === 'folder'}
    <DeleteDialog
      title="Delete folder {row.name}?"
      body={actions.contents(row.item.path)}
      confirmWord={row.name}
      button="Delete folder"
      ondelete={() => confirmDelete(row)}
      onclose={() => (deleting = null)} />
  {:else}
    <DeleteDialog
      title="Delete {row.name}?"
      body="The file is deleted from disk. Tags and attachment links on it are removed."
      ondelete={() => confirmDelete(row)}
      onclose={() => (deleting = null)} />
  {/if}
{/if}

{#if drag}
  <div class="ghost" style:left="{drag.x}px" style:top="{drag.y}px">{drag.row.name}{drag.row.isFolder ? '/' : ''}</div>
{/if}

<style>
  .explorer {
    position: relative;
    flex: 0 0 var(--explorer-w);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .toolbar {
    flex: 0 0 40px;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 0 8px;
  }

  .tree {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 0 0 16px;
    display: flex;
    flex-direction: column;
    user-select: none;
  }

  .row {
    position: relative;
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 0 0 28px;
    padding-right: 6px;
    font: 400 14px/20px var(--font-ui);
    color: var(--text);
    cursor: pointer;
    touch-action: none;
  }

  .row.leaf {
    color: var(--text-muted);
  }

  .row:hover {
    background: var(--c-hover);
  }

  .row.sel {
    background: var(--c-selected);
    color: var(--text);
  }

  .row.drop,
  .dragging .row.drop {
    background: var(--primary-soft);
    box-shadow: inset 0 0 0 1px var(--primary);
  }

  .row.bad {
    background: var(--danger-soft);
  }

  .row.ghosted {
    opacity: 0.4;
  }

  .dragging .row:hover:not(.drop):not(.bad) {
    background: transparent;
  }

  .chev {
    flex: 0 0 10px;
    width: 10px;
    height: 28px;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--c-muted-2);
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .nm {
    flex: 1;
    min-width: 0;
    height: 28px;
    padding: 0;
    border: 0;
    background: transparent;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: inherit;
    color: inherit;
    cursor: pointer;
    line-height: 28px;
  }

  .more {
    width: 22px;
    height: 22px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--c-muted-2);
    display: grid;
    place-items: center;
    cursor: pointer;
    visibility: hidden;
  }

  .row:hover .more,
  .more.on,
  .more:focus-visible {
    visibility: visible;
  }

  .more:hover,
  .more.on {
    background: var(--border);
    color: var(--text);
  }

  .dragging .more {
    visibility: hidden;
  }

  .ein {
    flex: 1;
    min-width: 0;
    height: 24px;
    padding: 0 6px;
    margin-left: -6px;
    border: 1px solid var(--primary);
    border-radius: var(--radius-sm);
    outline: none;
    background: var(--c-page);
    font: 400 14px/20px var(--font-ui);
    color: var(--text);
  }

  .ein.bad {
    border-color: var(--danger);
    color: var(--danger);
  }

  .name-err {
    padding: 2px 8px 6px;
    font: 400 12px/16px var(--font-ui);
    color: var(--danger);
  }

  .menu {
    min-width: 180px;
  }

  .menu .opt {
    height: 30px;
    font-size: 14px;
  }

  .caret {
    position: absolute;
    right: -8px;
    top: calc(50% - 14px);
    z-index: 2;
    width: 14px;
    height: 28px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--c-page);
    color: var(--text-muted);
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .caret.tab {
    position: fixed;
    left: 0;
    right: auto;
    top: calc(50% - 36px);
    border-left: 0;
    border-radius: 0 4px 4px 0;
  }

  .ghost {
    position: fixed;
    z-index: 60;
    pointer-events: none;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    background: var(--text);
    color: var(--c-page);
    font: 400 12px/16px var(--font-ui);
    white-space: nowrap;
  }
</style>
