<script lang="ts">
  // Attachments of one owner: a drop zone (or click to choose), "link a note", and the rows.
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import Icon from './Icon.svelte';
  import Popover from './Popover.svelte';
  import SearchPicker from './SearchPicker.svelte';
  import { attachmentsAPI } from '#lib/api/attachments.ts';
  import type { AttachmentLink, Owner } from '#lib/types/attachment.type.ts';
  import type { NoteRow } from '#lib/types/note.type.ts';
  import { messageOf } from '#lib/api/core.ts';
  import { notes } from '#lib/stores/notes.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { bytesLabel } from '#lib/util/bytes.ts';
  import { shortDate, iso } from '#lib/util/dates.ts';
  import { routes } from '#lib/util/routes.ts';

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
    try {
      await attachmentsAPI.add({ owner, id, paths });
      toast.show(paths.length === 1 ? 'File attached' : `${paths.length} files attached`);
    } catch (e) {
      toast.show(messageOf(e));
    }
    await load();
  }

  async function choose() {
    const picked = await openDialog({ multiple: true, title: 'Attach files' });
    if (picked) await addPaths(Array.isArray(picked) ? picked : [picked]);
  }

  // Files dragged in from the OS arrive as Tauri drag-drop events with paths.
  onMount(() => {
    if (!('__TAURI_INTERNALS__' in window)) return;
    const inside = (p: { x: number; y: number }) => {
      if (!zone) return false;
      const r = zone.getBoundingClientRect();
      const [x, y] = [p.x / devicePixelRatio, p.y / devicePixelRatio];
      return x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
    };
    const stop = getCurrentWebview().onDragDropEvent((e) => {
      const p = e.payload;
      if (p.type === 'over' || p.type === 'enter') over = inside(p.position);
      else if (p.type === 'leave') over = false;
      else if (p.type === 'drop') {
        const hit = inside(p.position);
        over = false;
        if (hit) addPaths(p.paths);
      }
    });
    return () => {
      stop.then((f) => f());
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

  async function openLink(l: AttachmentLink) {
    if (l.kind === 'note' && l.noteId) goto(routes.note(l.noteId));
    else if (l.attachmentId) await attachmentsAPI.open(l.attachmentId).catch((e) => toast.show(messageOf(e)));
  }

  const isImage = (l: AttachmentLink) => l.mimeType?.startsWith('image/') ?? false;
  const ext = (l: AttachmentLink) => (l.name.includes('.') ? l.name.slice(l.name.lastIndexOf('.') + 1).toUpperCase() : 'FILE');

  function meta(l: AttachmentLink): string {
    if (l.kind === 'note') return `note · ${l.folder || 'notes'}`;
    return `${ext(l)} · ${bytesLabel(l.sizeBytes ?? 0)} · ${shortDate(iso(new Date(l.createdAt)))}`;
  }

  const matchNote = (n: NoteRow, q: string) => !q || n.title.toLowerCase().includes(q) || n.name.includes(q) || n.folder.includes(q);

  async function startLink() {
    if (!notes.notes.length) await notes.load();
    picking = true;
  }
</script>

<div class="drop" class:over bind:this={zone} role="button" tabindex="0" onclick={choose} onkeydown={(e) => e.key === 'Enter' && choose()}>
  <span class="ic"><Icon name="upload" size={14} /></span>
  <span>Drop files here or click to choose</span>
  <span class="grow"></span>
  <span class="or">or <button bind:this={linkBtn} class="lnk" onclick={(e) => (e.stopPropagation(), startLink())}>link a note</button></span>
</div>
{#if picking}
  <Popover anchor={linkBtn} placement="bottom-end" onclose={() => (picking = false)}>
    <SearchPicker items={notes.notes} match={matchNote} onpick={linkNote} placeholder="Search notes…" label="Notes" empty="No notes found">
      {#snippet row(n)}
        <span class="note-ic"><Icon name="note" size={14} /></span>{n.title}<span class="sub">{n.folder}</span>
      {/snippet}
    </SearchPicker>
  </Popover>
{/if}

{#each links as l (l.id)}
  <div class="arow reveal-on-hover">
    <span class="ic" class:note={l.kind === 'note'}>
      <Icon name={l.kind === 'note' ? 'note' : isImage(l) ? 'image' : 'file'} size={14} />
    </span>
    <span class="an">{l.name}</span>
    <span class="am">{meta(l)}</span>
    <span class="acts on-hover">
      <button class="aib" aria-label="Open" title="Open" onclick={() => openLink(l)}><Icon name="open" size={14} /></button>
      {#if l.kind === 'file' && l.attachmentId}
        <button class="aib" aria-label="Show in folder" title="Show in folder" onclick={() => attachmentsAPI.reveal(l.attachmentId!)}>
          <Icon name="folder" size={14} />
        </button>
      {/if}
      <button class="aib del" aria-label="Remove" title="Remove" onclick={() => unlink(l)}><Icon name="close" size={12} stroke={2.25} /></button>
    </span>
  </div>
{/each}

<style>
  .drop {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 40px;
    margin: 2px 0 6px;
    padding: 0 12px;
    border: 1px dashed var(--border);
    border-radius: 6px;
    font: 400 13px/18px var(--font-ui);
    color: var(--c-muted-2);
    cursor: pointer;
  }

  .drop:hover {
    border-color: var(--c-faint);
    color: var(--text);
  }

  .drop.over {
    border-color: var(--primary);
    background: var(--primary-soft);
    color: var(--primary);
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

  .ic.note,
  .note-ic {
    display: grid;
    color: var(--accent);
  }

  .arow {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 36px;
    padding: 0 10px 0 16px;
    margin: 0 -16px;
    font: 400 14px/20px var(--font-ui);
  }

  .arow:hover {
    background: var(--c-hover);
  }

  .an {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .am {
    font: 400 12px/16px var(--font-ui);
    color: var(--c-muted-2);
  }

  .acts {
    display: flex;
    gap: 2px;
  }

  .aib {
    width: 24px;
    height: 24px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--c-muted-2);
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .aib:hover {
    background: var(--border);
    color: var(--text);
  }

  .aib.del {
    color: var(--danger);
  }

  .aib.del:hover {
    background: var(--danger-soft);
  }
</style>
