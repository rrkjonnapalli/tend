<script lang="ts">
  // Attachments of one owner: a drop zone (or click to choose), "link a note", and the rows.
  import { onMount } from 'svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { getCurrentWebview, type DragDropEvent } from '@tauri-apps/api/webview';
  import Icon from './Icon.svelte';
  import Popover from './Popover.svelte';
  import SearchPicker from './SearchPicker.svelte';
  import AttachmentRow from './AttachmentRow.svelte';
  import { attachmentsAPI } from '#lib/api/attachments.ts';
  import type { AttachmentLink, Owner } from '#lib/types/attachment.type.ts';
  import type { NoteRow } from '#lib/types/note.type.ts';
  import { notes } from '#lib/stores/notes.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';

  let { owner, id, onchange }: { owner: Owner; id: string; onchange?: (count: number) => void } = $props();

  let links = $state<AttachmentLink[]>([]);
  let zone = $state<HTMLElement>();
  let over = $state(false);
  let picking = $state(false);
  let linkBtn = $state<HTMLButtonElement>();

  async function load() {
    links = await attachmentsAPI.list({ owner, id });
    onchange?.(links.length);
  }

  $effect(() => {
    void id;
    load();
  });

  async function addPaths(paths: string[]) {
    if (!paths.length) return;
    // Adding stops at the first file that fails; reload either way to show the ones that made it.
    try {
      await attachmentsAPI.add({ owner, id, paths });
      toast.show(paths.length === 1 ? 'File attached' : `${paths.length} files attached`);
    } finally {
      await load();
    }
  }

  async function choose() {
    const picked = await openDialog({ multiple: true, title: 'Attach files' });
    if (!picked) return;
    await addPaths(Array.isArray(picked) ? picked : [picked]);
  }

  function zoneKeys(e: KeyboardEvent) {
    if (e.key === 'Enter') choose();
  }

  // Files dragged in from the OS arrive as Tauri drag-drop events with paths, in physical pixels.
  function isOverZone(p: { x: number; y: number }): boolean {
    if (!zone) return false;
    const r = zone.getBoundingClientRect();
    const x = p.x / devicePixelRatio;
    const y = p.y / devicePixelRatio;
    return x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
  }

  function onDragDrop(p: DragDropEvent) {
    if (p.type === 'over' || p.type === 'enter') {
      over = isOverZone(p.position);
      return;
    }
    if (p.type === 'leave') {
      over = false;
      return;
    }
    const hit = isOverZone(p.position);
    over = false;
    if (hit) addPaths(p.paths);
  }

  const inTauri = () => '__TAURI_INTERNALS__' in window;

  onMount(() => {
    if (!inTauri()) return;
    const stop = getCurrentWebview().onDragDropEvent((e) => onDragDrop(e.payload));
    return () => {
      stop.then((unlisten) => unlisten());
    };
  });

  async function linkNote(n: NoteRow) {
    picking = false;
    await attachmentsAPI.linkNote({ owner, id, note: n.id });
    await load();
  }

  async function unlink(l: AttachmentLink) {
    await attachmentsAPI.unlink(l.id);
    await load();
  }

  const matchNote = (n: NoteRow, q: string) => !q || n.name.includes(q) || n.folder.includes(q);

  async function startLink(e: MouseEvent) {
    // The button sits inside the drop zone, whose click opens the file dialog.
    e.stopPropagation();
    if (!notes.notes.length) await notes.load();
    picking = true;
  }
</script>

<div data-zone="bar">
  <div class="drop" class:over bind:this={zone} role="button" tabindex="0" onclick={choose} onkeydown={zoneKeys}>
    <span class="ic"><Icon name="upload" size={14} /></span>
    <span>Drop files here or click to choose</span>
    <span class="grow"></span>
    <span class="or">or <button bind:this={linkBtn} class="lnk" onclick={startLink}>link a note</button></span>
  </div>
  {#if picking}
    <Popover anchor={linkBtn} placement="bottom-end" onclose={() => (picking = false)}>
      <SearchPicker items={notes.notes} match={matchNote} onpick={linkNote} placeholder="Search notes…" label="Notes" empty="No notes found">
        {#snippet row(n)}
          <span class="note-ic"><Icon name="note" size={14} /></span>{n.name}<span class="sub">{n.folder}</span>
        {/snippet}
      </SearchPicker>
    </Popover>
  {/if}
</div>
<div data-zone="rows">
  {#each links as l (l.id)}
    <AttachmentRow link={l} onremove={() => unlink(l)} />
  {/each}
</div>

<style>
  .drop {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 40px;
    margin: 2px 0 6px;
    padding: 0 12px;
    border: 1px dashed var(--border);
    border-radius: var(--radius-control);
    font: var(--text-caption);
    color: var(--c-muted-2);
    cursor: pointer;
  }

  .drop:hover {
    border-color: var(--c-faint);
    color: var(--text);
  }

  .drop.over {
    border-color: var(--c-current);
    background: var(--c-selected);
    color: var(--c-current-ink);
  }

  .grow {
    flex: 1;
  }

  .or {
    color: var(--c-muted-2);
  }

  .lnk {
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--link);
    font: inherit;
    cursor: pointer;
  }

  .lnk:hover {
    text-decoration: underline;
  }

  .ic {
    display: grid;
    color: var(--c-muted-2);
  }

  .note-ic {
    display: grid;
    color: var(--text-muted);
  }
</style>
