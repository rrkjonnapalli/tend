<script lang="ts">
  // The snap editor and the snap delete confirm, shown over any page: ⌘S opens a blank paper
  // anywhere, and the Snaps board opens the same one. A new snap goes in the stack the board
  // shows, or loose on the board when another page is open.
  import { page } from '$app/state';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import SnapEditor from './SnapEditor.svelte';
  import { snapsAPI } from '#lib/api/snaps.ts';
  import { LIBRARY_AREA } from '#lib/enums/library.enum.ts';
  import { PAPER } from '#lib/enums/snap.enum.ts';
  import { snaps } from '#lib/stores/snaps.svelte.ts';
  import { tagPool } from '#lib/stores/tags.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import type { SavedDraft, Snap, SnapDraft } from '#lib/types/snap.type.ts';
  import { routes, STACK_PARAM } from '#lib/util/routes.ts';
  import { NEW_STACK, whereLabel } from '#lib/util/snaps.ts';

  const onBoard = $derived(page.url.pathname === routes.libraryArea(LIBRARY_AREA.snaps));
  /** The stack the board shows; none on the board itself and on every other page. */
  const open = $derived(onBoard ? snaps.stackOr(page.url.searchParams.get(STACK_PARAM)) : null);

  const where = (id: string | null) => whereLabel(id, snaps.stacks);

  // The stack picker lists every stack: off the board they may not be loaded yet.
  $effect(() => {
    if (snaps.editing && !snaps.loaded) snaps.load();
  });

  /** The stack a saved snap goes in: the one picked, or a new one where the board is. */
  async function targetOf(draft: SnapDraft): Promise<string | null> {
    if (draft.stackId !== NEW_STACK) return draft.stackId;
    const made = await snapsAPI.createStack({ parent: open, name: '' });
    return made.id;
  }

  /** Writes the draft (a new stack first, then the snap, then its tags); resolves to its stack. */
  async function writeDraft(draft: SnapDraft, before: Snap | null): Promise<SavedDraft> {
    const stackId = await targetOf(draft);
    const color = draft.paper === PAPER.sticky ? draft.color : null;
    const input = { text: draft.text, stackId, paper: draft.paper, color, pinned: draft.pinned };
    const saved = before ? await snapsAPI.update({ id: before.id, patch: input }) : await snapsAPI.create(input);
    return { stackId, tagsError: await writeTags(saved, draft.tags) };
  }

  /** Saves the tags when they changed. A refusal (a tag blocked since the pool was loaded) is
   *  returned, not thrown: the snap is saved by then, and the editor closes first. */
  async function writeTags(saved: Snap, tags: string[]): Promise<unknown> {
    const tagsChanged = tags.join(' ') !== saved.tags.join(' ');
    if (!tagsChanged) return null;
    try {
      await snapsAPI.setTags({ id: saved.id, tags });
      return null;
    } catch (e) {
      return e;
    }
  }

  /** What the save says: where a new snap went, or where an edited one moved. */
  function savedMessage(before: Snap | null, stackId: string | null): string {
    if (!before) return `Added to ${where(stackId)}`;
    if (before.stackId !== stackId) return `Saved and moved to ${where(stackId)}`;
    return 'Saved';
  }

  /** A stack made by the save is named right there on the board; elsewhere it stays untitled. */
  function announce(draft: SnapDraft, before: Snap | null, stackId: string | null) {
    const madeStack = draft.stackId === NEW_STACK;
    if (madeStack && onBoard) snaps.naming = stackId;
    else toast.show(savedMessage(before, stackId));
  }

  let saving = false;

  async function saveDraft(draft: SnapDraft) {
    // A second ⌘↵ while the first save runs would write the snap twice.
    if (saving) return;
    saving = true;
    const before = snaps.editing?.snap ?? null;
    let written: SavedDraft;
    try {
      written = await writeDraft(draft, before);
    } finally {
      saving = false;
      // shortcut: a save that fails halfway can leave a new empty stack; the reload shows it. One Rust command would make it all-or-nothing.
      await snaps.load();
    }
    snaps.editing = null;
    // Saved without the new tags: closed, so no refused chip stays and a retry can't add it twice.
    if (written.tagsError) {
      await tagPool.load();
      throw written.tagsError;
    }
    announce(draft, before, written.stackId);
  }

  function deleteEditing() {
    const snap = snaps.editing?.snap;
    if (snap) snaps.askDelete(snap);
  }
</script>

{#if snaps.editing}
  {#key snaps.editing}
    <SnapEditor
      snap={snaps.editing.snap}
      filled={snaps.editing.filled}
      {open}
      onsave={saveDraft}
      ondelete={deleteEditing}
      onclose={() => (snaps.editing = null)} />
  {/key}
{/if}

{#if snaps.deleting}
  {@const snap = snaps.deleting}
  <ConfirmDialog
    title="Delete this snap?"
    body="It goes from the board."
    confirm="Delete"
    danger
    onconfirm={() => snaps.remove(snap)}
    onclose={() => (snaps.deleting = null)} />
{/if}
