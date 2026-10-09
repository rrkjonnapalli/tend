<script lang="ts" module>
  // Expanded folders per explorer, kept while the app runs so switching sections keeps them.
  const expandedBy = new Map<string, Record<string, boolean>>();
</script>

<script lang="ts">
  // The left-hand tree shared by Notes (folders + notes) and Tasks (folders only).
  import type { DropState, ExplorerActions, ExplorerEdit, ExplorerItem, ExplorerRow, Leaf, NewKind } from '#lib/types/explorer.type.ts';
  import type { Anchor } from '#lib/types/popover.type.ts';
  import { untrack } from 'svelte';
  import Icon from './Icon.svelte';
  import ExplorerRowView from './ExplorerRow.svelte';
  import ExplorerMenu from './ExplorerMenu.svelte';
  import MoveToDialog from './MoveToDialog.svelte';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import { baseOf, join, nameError, nameErrors, parentOf, titleError } from '#lib/util/names.ts';
  import { isTypingTarget, matches } from '#lib/util/keys.ts';
  import { hasLayers } from '#lib/util/layers.ts';
  import {
    NEW_KEY,
    ROOT_KEY,
    buildRows,
    canDrop,
    folderPath,
    isRoot,
    keyOf,
    withAncestors,
  } from '#lib/util/explorer-rows.ts';
  import { dropTargetAt, trackDrag } from '#lib/util/explorer-drag.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { rowNav } from '#lib/actions/row-nav.action.ts';

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

  /** The drag ghost sits just below and right of the pointer. */
  const GHOST_OFFSET_X = 12;
  const GHOST_OFFSET_Y = 8;

  // The explorer id never changes for a mounted explorer.
  const memoryKey = untrack(() => id);
  let expanded = $state<Record<string, boolean>>(expandedBy.get(memoryKey) ?? { '': true });
  $effect(() => {
    expandedBy.set(memoryKey, expanded);
  });

  let editing = $state<ExplorerEdit | null>(null);
  let draft = $state('');
  let menu = $state<{ row: ExplorerRow; anchor: Anchor } | null>(null);
  let moving = $state<ExplorerRow | null>(null);
  let deleting = $state<ExplorerRow | null>(null);
  let drag = $state<{ row: ExplorerRow; over: string | null; x: number; y: number } | null>(null);
  let suppressClick = false;

  const tree = $derived({ folders, leaves });
  const selectedKey = $derived(selected ? keyOf(selected) : null);
  const selectedFolder = $derived.by(() => {
    if (!selected) return '';
    if (selected.kind === 'folder') return selected.path;
    return leaves.find((l) => l.id === selected.id)?.folder ?? '';
  });

  // Opening an item reveals it: every folder above it expands.
  $effect(() => {
    const start = selected?.kind === 'folder' ? parentOf(selected.path) : selectedFolder;
    const hidden = withAncestors(start).filter((path) => !expanded[path]);
    if (hidden.length === 0) return;
    const next = { ...expanded };
    for (const path of hidden) next[path] = true;
    expanded = next;
  });

  const adding = $derived.by(() => {
    if (!editing?.isNew) return null;
    return { kind: editing.isNew, folder: editing.folder };
  });
  const rows = $derived(buildRows({ tree, rootName, expanded, adding }));
  const selectedRow = $derived(rows.find((r) => r.key === selectedKey && !isRoot(r)));
  const allFolders = $derived(['', ...folders]);

  const hasMenu = (row: ExplorerRow) => !row.isNew && (!isRoot(row) || !!actions.renameRoot);
  const allowsDrop = (row: ExplorerRow, target: string) => canDrop({ row, target });

  // The root's display name is free text; a new note takes a title (its file gets the slug);
  // renamed notes and folders become slugs.
  const draftError = $derived.by(() => {
    if (!editing) return '';
    if (editing.key === ROOT_KEY) return draft.trim() ? '' : nameErrors.empty;
    return editing.isNew === 'leaf' ? titleError(draft) : nameError(draft);
  });

  // ---- naming

  function startRename(row: ExplorerRow) {
    menu = null;
    editing = { key: row.key, folder: row.folder, item: row.item };
    draft = row.name;
  }

  function startNew(kind: NewKind) {
    const folder = selectedFolder;
    expanded = { ...expanded, [folder]: true };
    editing = { key: NEW_KEY, folder, isNew: kind };
    draft = '';
  }

  function onNewItem() {
    if (actions.newItem) actions.newItem(selectedFolder);
    else startNew('leaf');
  }

  async function createFolder(parent: string, name: string) {
    const path = await actions.createFolder({ parent, name });
    toast.show(`Folder ${path} created`);
  }

  async function renameRoot(name: string) {
    if (name !== rootName) await actions.renameRoot?.(name);
  }

  function currentName(item: ExplorerItem): string | undefined {
    if (item.kind === 'folder') return baseOf(item.path);
    return leaves.find((l) => l.id === item.id)?.name;
  }

  async function renameItem(item: ExplorerItem, name: string) {
    if (currentName(item) === name) return;
    await actions.rename(item, name);
    toast.show('Renamed');
  }

  async function save(edit: ExplorerEdit, name: string) {
    if (edit.isNew === 'folder') await createFolder(edit.folder, name);
    else if (edit.isNew === 'leaf') await actions.createLeaf?.({ folder: edit.folder, name });
    else if (edit.key === ROOT_KEY) await renameRoot(name);
    else if (edit.item) await renameItem(edit.item, name);
  }

  async function commit() {
    const edit = editing;
    if (!edit) return;
    const name = draft.trim();
    if (draftError) {
      // An empty new row is dropped; a bad name stays open for fixing.
      if (edit.isNew && !name) editing = null;
      return;
    }
    editing = null;
    await save(edit, name);
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

  async function moveTo(row: ExplorerRow, target: string) {
    moving = null;
    await actions.move(row.item, target);
    expanded = { ...expanded, [target]: true };
    toast.show(`Moved to ${target || rootName}/`);
  }

  function canDrag(row: ExplorerRow, e: PointerEvent): boolean {
    const isPrimary = e.button === 0;
    const isEditing = editing?.key === row.key;
    return isPrimary && !row.isNew && !isEditing && !isRoot(row);
  }

  const ghostAt = (e: PointerEvent) => ({ x: e.clientX + GHOST_OFFSET_X, y: e.clientY + GHOST_OFFSET_Y });

  /** The click that ends a drag must not also open the row. */
  function suppressNextClick() {
    suppressClick = true;
    setTimeout(() => {
      suppressClick = false;
    }, 0);
  }

  function endDrag() {
    const done = drag;
    drag = null;
    if (!done) return;
    suppressNextClick();
    const target = done.over;
    if (target !== null && allowsDrop(done.row, target)) moveTo(done.row, target);
  }

  function onRowDown(row: ExplorerRow, e: PointerEvent) {
    if (!canDrag(row, e)) return;
    trackDrag({
      start: e,
      onstart: (ev) => {
        menu = null;
        drag = { row, over: null, ...ghostAt(ev) };
      },
      onmove: (ev) => {
        if (drag) drag = { ...drag, over: dropTargetAt(ev), ...ghostAt(ev) };
      },
      onend: endDrag,
    });
  }

  function dropStateOf(row: ExplorerRow): DropState {
    if (drag === null || drag.over === null || row.isNew) return null;
    if (drag.over !== folderPath(row.item)) return null;
    return allowsDrop(drag.row, drag.over) ? 'allowed' : 'blocked';
  }

  // ---- deleting

  /** "work/meetings/" for a folder, the bare name for a leaf. */
  function displayName(row: ExplorerRow): string {
    if (row.item.kind === 'folder') return `${row.item.path}/`;
    return row.name;
  }

  async function confirmDelete(row: ExplorerRow) {
    deleting = null;
    await actions.remove(row.item);
    toast.show(`Deleted ${displayName(row)}`);
  }

  function deleteCopy(row: ExplorerRow) {
    if (row.item.kind === 'folder') {
      return {
        title: `Delete folder ${row.name}?`,
        body: actions.contents(row.item.path),
        confirmWord: row.name,
        confirm: 'Delete folder',
      };
    }
    return {
      title: `Delete ${row.name}?`,
      body: 'The file is deleted from disk. Tags and attachment links on it are removed.',
      confirm: 'Delete',
    };
  }

  // ---- interactions

  function openRow(row: ExplorerRow) {
    if (suppressClick || row.isNew || editing?.key === row.key) return;
    menu = null;
    actions.open(row.item);
  }

  function toggle(path: string, e: Event) {
    e.stopPropagation();
    expanded = { ...expanded, [path]: !expanded[path] };
  }

  function collapseAll() {
    expanded = { '': true };
  }

  function openMenu(row: ExplorerRow, e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    if (!hasMenu(row)) return;
    const anchor = e.type === 'contextmenu' ? { x: e.clientX, y: e.clientY } : (e.currentTarget as HTMLElement);
    menu = { row, anchor };
  }

  function askMove(row: ExplorerRow) {
    moving = row;
    menu = null;
  }

  function askDelete(row: ExplorerRow) {
    deleting = row;
    menu = null;
  }

  /** The row with the keyboard cursor, when focus is in the tree. */
  function focusedRow(target: EventTarget | null): ExplorerRow | undefined {
    if (!(target instanceof HTMLElement)) return undefined;
    const key = target.closest<HTMLElement>('.tree [data-row]')?.dataset.key;
    return rows.find((r) => r.key === key && !isRoot(r));
  }

  /** F2, ⌘⇧M and Backspace act on the explorer row with the cursor, else on the open item's row. */
  function onkeydown(e: KeyboardEvent) {
    const busy = hasLayers() || editing !== null || isTypingTarget(e.target);
    // A list row with the cursor (notes, tasks, people) handles its own keys.
    const inList = e.target instanceof HTMLElement && e.target.closest('[data-row]:not(.tree [data-row])') !== null;
    if (e.defaultPrevented || inList) return;
    const row = focusedRow(e.target) ?? selectedRow;
    if (!ui.explorerOpen || busy || !row) return;
    if (e.key === 'F2') {
      e.preventDefault();
      startRename(row);
    } else if (matches(e, 'mod+shift+m')) {
      e.preventDefault();
      moving = row;
    } else if (e.key === 'Backspace' && !e.metaKey && !e.ctrlKey) {
      e.preventDefault();
      deleting = row;
    }
  }
</script>

<svelte:window {onkeydown} />

{#if ui.explorerOpen}
  <aside class="explorer" class:dragging={drag !== null} aria-label="Explorer">
    <div class="toolbar" role="toolbar" aria-label="Explorer actions">
      <button class="icon-btn sm" aria-label={newItemLabel} title={newItemLabel} onclick={onNewItem}>
        <Icon name={actions.newItem ? 'taskNew' : 'noteNew'} size={13} stroke={1.75} />
      </button>
      <button class="icon-btn sm" aria-label="New folder" title="New folder" onclick={() => startNew('folder')}>
        <Icon name="folderNew" size={13} stroke={1.75} />
      </button>
      <button class="icon-btn sm" aria-label="Reload" title="Reload" onclick={() => actions.reload()}>
        <Icon name="reload" size={13} stroke={1.75} />
      </button>
      <button class="icon-btn sm" aria-label="Collapse all" title="Collapse all" onclick={collapseAll}>
        <Icon name="collapseAll" size={13} stroke={1.75} />
      </button>
    </div>

    <div class="tree" role="tree" use:rowNav>
      {#each rows as row (row.key)}
        {@const path = folderPath(row.item) ?? ''}
        <ExplorerRowView
          {row}
          selected={row.key === selectedKey && !drag}
          drop={dropStateOf(row)}
          ghosted={drag?.row.key === row.key}
          expanded={!!expanded[path]}
          editing={editing?.key === row.key}
          bind:draft
          {draftError}
          hasMenu={hasMenu(row)}
          menuOpen={menu?.row.key === row.key}
          onpress={(e) => onRowDown(row, e)}
          onmenu={(e) => openMenu(row, e)}
          onopen={() => openRow(row)}
          ontoggle={(e) => toggle(path, e)}
          oneditkey={onEditKey}
          oneditblur={onEditBlur} />
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
  <ExplorerMenu
    anchor={menu.anchor}
    isRoot={isRoot(row)}
    onrename={() => startRename(row)}
    onmove={() => askMove(row)}
    ondelete={() => askDelete(row)}
    onclose={() => (menu = null)} />
{/if}

{#if moving}
  {@const row = moving}
  <MoveToDialog
    what={row.item.kind === 'folder' ? `${row.item.path}/` : join(row.folder, row.name)}
    folders={allFolders}
    current={row.folder}
    self={folderPath(row.item)}
    {rootName}
    onmove={(target) => moveTo(row, target)}
    onclose={() => (moving = null)} />
{/if}

{#if deleting}
  {@const row = deleting}
  <ConfirmDialog {...deleteCopy(row)} danger onconfirm={() => confirmDelete(row)} onclose={() => (deleting = null)} />
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
    font: var(--text-meta);
    white-space: nowrap;
  }
</style>
