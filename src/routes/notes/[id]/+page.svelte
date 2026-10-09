<script lang="ts">
  import { settingKeys } from '#lib/config/settings.config.ts';
  // Note page: sticky bar (back, breadcrumb, width, read/edit), tags, then the note itself.
  // Edits autosave to disk ~500 ms after typing stops, and on blur or leaving the page.
  import type { Crumb } from '#lib/types/breadcrumb.type.ts';
  import type { NoteDoc } from '#lib/types/note.type.ts';
  import { beforeNavigate } from '$app/navigation';
  import { page } from '$app/state';
  import Icon from '#lib/components/Icon.svelte';
  import Breadcrumbs from '#lib/components/Breadcrumbs.svelte';
  import PageBar from '#lib/components/PageBar.svelte';
  import TagEditor from '#lib/components/TagEditor.svelte';
  import MarkdownView from '#lib/components/MarkdownView.svelte';
  import MarkdownEditor from '#lib/components/MarkdownEditor.svelte';
  import { notesAPI } from '#lib/api/notes.ts';
  import { tagsAPI } from '#lib/api/tags.ts';
  import { notes } from '#lib/stores/notes.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { label as keyLabel } from '#lib/util/keys.ts';
  import { pageActions } from '#lib/stores/page-actions.svelte.ts';
  import { routes } from '#lib/util/routes.ts';
  import { folderCrumbs } from '#lib/util/note-paths.ts';
  import { nav } from '#lib/stores/history.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { toAppError } from '#lib/errors/app-error.ts';

  const SAVE_MS = 500;
  const EDITOR_MIN_HEIGHT = 600;

  const id = $derived(page.params.id!);
  const row = $derived(notes.byId(id));
  /** The open note's folder, kept after the note leaves the tree (deleted outside Tend). */
  let lastFolder = $state('');
  $effect(() => {
    if (row) lastFolder = row.folder;
  });

  let text = $state('');
  let saved = $state('');
  let tags = $state<string[]>([]);
  let editing = $state(false);
  let loadedId = $state<string | null>(null);
  let timer: ReturnType<typeof setTimeout> | undefined;

  const wide = $derived(settings.get(settingKeys.notesWide, false));
  const widthLabel = $derived(wide ? 'Narrow width' : 'Full width');

  function show({ noteId, doc, edit }: { noteId: string; doc: NoteDoc; edit: boolean }) {
    text = doc.text;
    saved = doc.text;
    tags = doc.tags;
    loadedId = noteId;
    editing = edit;
  }

  async function load({ noteId, edit }: { noteId: string; edit: boolean }) {
    const doc = await notesAPI.read(noteId);
    show({ noteId, doc, edit });
  }

  // Open the note when the id changes.
  $effect(() => {
    const noteId = id;
    const edit = page.url.searchParams.get('edit') === '1';
    if (noteId !== loadedId) {
      flush();
      load({ noteId, edit });
    }
  });

  /** True while the open note has no edits of its own that a reload would overwrite. */
  const isUntouched = () => !editing && text === saved;

  function takeOutsideText(doc: NoteDoc) {
    if (!isUntouched() || doc.text === text) return;
    text = doc.text;
    saved = doc.text;
  }

  /** The open note was deleted outside Tend: go back to its folder. */
  function leaveIfGone(e: unknown) {
    if (!toAppError(e).is('NOT_FOUND')) throw e;
    toast.show('This note was deleted outside Tend');
    nav.replace(routes.notesFolder(lastFolder));
  }

  // Outside changes reach an open note that isn't being edited.
  $effect(() => {
    void notes.version;
    if (loadedId === id && isUntouched()) {
      notesAPI.read(id).then(takeOutsideText, leaveIfGone);
    }
  });

  async function flush() {
    clearTimeout(timer);
    if (loadedId === null || text === saved) return;
    const noteId = loadedId;
    const body = text;
    await notesAPI.write({ id: noteId, text: body });
    if (loadedId === noteId) saved = body;
  }

  function onchange(next: string) {
    text = next;
    clearTimeout(timer);
    timer = setTimeout(flush, SAVE_MS);
  }

  function setMode(edit: boolean) {
    if (!edit) flush();
    editing = edit;
  }

  async function setTags(next: string[]) {
    tags = next;
    tags = await tagsAPI.setForNote({ id, tags: next });
  }

  beforeNavigate(() => {
    flush();
  });

  let tagEditor = $state<TagEditor>();

  $effect(() =>
    pageActions.provide({
      'note.mode': () => setMode(!editing),
      'note.tag': () => tagEditor?.start(),
      'page.width': toggleWidth,
    }),
  );

  function toggleWidth() {
    settings.set(settingKeys.notesWide, !wide);
  }

  const crumbs = $derived.by((): Crumb[] => {
    const trail = folderCrumbs(row?.folder ?? '');
    if (!row) return trail;
    return [...trail, { label: row.name }];
  });
</script>

<main class="main">
  <PageBar onback={() => nav.back(routes.notesFolder(row?.folder ?? ''))}>
    <Breadcrumbs {crumbs} />
    {#snippet actions()}
    <button
      class="icon-btn"
      aria-label={widthLabel}
      title={widthLabel}
      onclick={toggleWidth}>
      <Icon name={wide ? 'narrow' : 'widen'} size={14} />
    </button>
    {#if editing}
      <button class="icon-btn" aria-label="Read" title="Read ({keyLabel('mod+r')})" onclick={() => setMode(false)}><Icon name="read" size={13} /></button>
    {:else}
      <button class="icon-btn" aria-label="Edit" title="Edit ({keyLabel('mod+r')})" onclick={() => setMode(true)}><Icon name="edit" size={13} /></button>
    {/if}
    {/snippet}
  </PageBar>

  {#if loadedId === id}
    <article class="note">
      <TagEditor bind:this={tagEditor} {tags} onchange={setTags} />
      <div class="body" style:max-width={wide ? 'none' : 'var(--content-w)'}>
        {#if editing}
          {#key id}
            <MarkdownEditor
              value={text}
              {onchange}
              onblur={flush}
              variant="note"
              minHeight={EDITOR_MIN_HEIGHT}
              placeholder="Start writing…"
              label="Note"
              autofocus />
          {/key}
        {:else}
          <div class="view">
            <MarkdownView {text} variant="note" empty="Empty note" ondblclick={() => setMode(true)} />
          </div>
        {/if}
      </div>
    </article>
  {/if}
</main>

<style>
  .main {
    flex: 1 1 auto;
    min-width: 0;
    padding: 0 24px 48px;
    overflow: auto;
  }


  .note {
    padding-top: 8px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .body {
    width: 100%;
    margin: 0 auto;
  }

  .view {
    min-height: 600px;
  }
</style>
