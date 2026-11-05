<script lang="ts">
  // What a phone list does with its selected rows (MD-Select): the action bar, the rename sheet for
  // one item, and the delete question (asked unless Settings turned it off for this kind).
  import ActionBar from './ActionBar.svelte';
  import ActionSheet from './ActionSheet.svelte';
  import Sheet from './Sheet.svelte';
  import { focusOnMount } from '#lib/actions/focus.action.ts';
  import { clip } from '#lib/stores/clip.svelte.ts';
  import { plural } from '#lib/util/plural.ts';

  let {
    folder,
    noun,
    asksDelete,
    nameOf,
    onrename,
    onremove,
  }: {
    folder: string;
    /** "note" or "task", for the questions. */
    noun: string;
    asksDelete: boolean;
    nameOf: (id: string) => string;
    onrename: (id: string, name: string) => Promise<unknown>;
    onremove: (id: string) => Promise<unknown>;
  } = $props();

  let renaming = $state<string | null>(null);
  let name = $state('');
  let deleting = $state(false);

  const ids = $derived(clip.selecting?.ids ?? []);

  function startRename() {
    renaming = ids[0];
    name = nameOf(ids[0]);
  }

  async function rename() {
    if (renaming) await onrename(renaming, name.trim());
    renaming = null;
    clip.stop();
  }

  async function remove() {
    for (const id of ids) await onremove(id);
    deleting = false;
    clip.stop();
  }

  function askDelete() {
    if (asksDelete) deleting = true;
    else remove();
  }
</script>

<ActionBar oncut={() => clip.cutSelected(folder)} onrename={startRename} ondelete={askDelete} />

{#if renaming}
  <Sheet title="Rename" action="Save" disabled={name.trim() === ''} onaction={rename} onclose={() => (renaming = null)}>
    <input use:focusOnMount class="p-input" aria-label="Name" autocapitalize="off" bind:value={name} />
  </Sheet>
{/if}

{#if deleting}
  <ActionSheet
    title={ids.length === 1 ? `Delete ${nameOf(ids[0])}?` : `Delete ${plural(ids.length, noun)}?`}
    body="They are removed from the vault on every device after the next sync. You can turn this question off in Settings › General."
    confirm={ids.length === 1 ? `Delete ${noun}` : `Delete ${plural(ids.length, noun)}`}
    danger
    onconfirm={remove}
    onclose={() => (deleting = false)} />
{/if}

