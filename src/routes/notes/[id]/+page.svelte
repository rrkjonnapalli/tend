<script lang="ts">
  import { settingDefaults, settingKeys } from '#lib/config/settings.config.ts';
  // Note page: sticky bar (back, breadcrumb, width, read/edit), tags, then the note itself.
  // Edits autosave to disk ~500 ms after typing stops, and on blur or leaving the page.
  import type { Crumb } from '#lib/types/breadcrumb.type.ts';
  import type { NoteDoc } from '#lib/types/note.type.ts';
  import { beforeNavigate } from '$app/navigation';
  import { untrack } from 'svelte';
  import { page } from '$app/state';
  import Icon from '#lib/components/Icon.svelte';
  import Breadcrumbs from '#lib/components/Breadcrumbs.svelte';
  import PageBar from '#lib/components/PageBar.svelte';
  import TagEditor from '#lib/components/TagEditor.svelte';
  import SaveStatus from '#lib/components/SaveStatus.svelte';
  import type { SaveState } from '#lib/types/save.type.ts';
  import DeletedDialog from '#lib/components/DeletedDialog.svelte';
  import { useDeleted } from '#lib/hooks/deleted.hook.svelte.ts';
  import { errors } from '#lib/errors/registry.ts';
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
  /** Deleted elsewhere: the page keeps the note and its text until the user keeps or discards it. */
  const deleted = useDeleted({ id: () => id, live: () => row });
  /** The note's place, still known while it is gone. */
  const place = $derived(row ?? deleted.last);

  let text = $state('');
  let saved = $state('');
  let tags = $state<string[]>([]);
  let editing = $state(false);
  let loadedId = $state<string | null>(null);
  let timer: ReturnType<typeof setTimeout> | undefined;
  /** A write is on its way to disk. */
  let writing = $state(false);
  /** The last write failed; the text stays on screen until a retry works. */
  let failed = $state(false);

  /** Edits not yet on disk count as saving: the autosave is at most SAVE_MS away. */
  const saveState = $derived.by((): SaveState => {
    if (failed) return 'failed';
    if (writing || text !== saved) return 'saving';
    return 'saved';
  });

  const wide = $derived(settings.get(settingKeys.notesWide, settingDefaults.notesWide));
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

  /** A note deleted elsewhere is noticed by `deleted` when the tree reloads; a read or save that
   *  races it just stops. */
  function ignoreMissing(e: unknown) {
    if (!toAppError(e).is(errors.NOT_FOUND)) throw e;
  }

  async function keep() {
    if (!deleted.last) return;
    const { folder, name } = deleted.last;
    const made = await notes.restore({ folder, name, text, tags });
    deleted.settle();
    toast.show('Kept as a new note');
    nav.replace(routes.note(made.id, { edit: editing }));
  }

  function discard() {
    deleted.settle();
    nav.replace(routes.notesFolder(place?.folder ?? ''));
  }

  // After outside changes, a note that isn't being edited takes the new text. The id is captured,
  // so a late answer for a note that was left never lands in the next one.
  $effect(() => {
    void notes.version;
    const noteId = untrack(() => loadedId);
    if (noteId === null) return;
    const takeIfStillOpen = (doc: NoteDoc) => {
      if (noteId === loadedId) takeOutsideText(doc);
    };
    notesAPI.read(noteId).then(takeIfStillOpen, ignoreMissing);
  });

  async function flush() {
    clearTimeout(timer);
    if (deleted.gone || loadedId === null || text === saved) return;
    const noteId = loadedId;
    const body = text;
    writing = true;
    try {
      await notesAPI.write({ id: noteId, text: body });
    } catch (e) {
      failed = !toAppError(e).is(errors.NOT_FOUND);
      ignoreMissing(e);
      return;
    } finally {
      writing = false;
    }
    failed = false;
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
    const trail = folderCrumbs(place?.folder ?? '');
    if (!place) return trail;
    return [...trail, { label: place.name }];
  });
</script>

<main class="detail-page detail-stack">
  <PageBar onback={() => nav.back(routes.notesFolder(place?.folder ?? ''))}>
    <Breadcrumbs {crumbs} />
    {#snippet actions()}
      {#if loadedId === id}<SaveStatus state={saveState} onretry={flush} />{/if}
      {#if editing}
        <button class="icon-btn" aria-label="Read" title="Read ({keyLabel('mod+r')})" onclick={() => setMode(false)}><Icon name="read" size={14} /></button>
      {:else}
        <button class="icon-btn" aria-label="Edit" title="Edit ({keyLabel('mod+r')})" onclick={() => setMode(true)}><Icon name="edit" size={14} /></button>
      {/if}
      <button class="icon-btn" aria-label={widthLabel} title={widthLabel} onclick={toggleWidth}>
        <Icon name={wide ? 'narrow' : 'widen'} size={14} />
      </button>
    {/snippet}
  </PageBar>

  {#if loadedId === id}
    <article class="detail-body note" style:max-width={wide ? 'none' : 'var(--content-w)'}>
      <TagEditor bind:this={tagEditor} {tags} onchange={setTags} />
      <div>
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

{#if deleted.gone}
  <DeletedDialog kind="note" onkeep={keep} ondiscard={discard} />
{/if}

<style>
  /* Tags sit just above the text. */
  .note {
    gap: 16px;
  }

  .view {
    min-height: 600px;
  }
</style>
