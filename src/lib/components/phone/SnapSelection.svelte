<script lang="ts">
  // What the board does with its selected snaps on a phone (MD-Snap-Select): Cut · Tag · Delete.
  // Tag adds tags to every selected snap; Delete asks first unless Settings turned that off.
  import ActionBar from './ActionBar.svelte';
  import ActionSheet from './ActionSheet.svelte';
  import Sheet from './Sheet.svelte';
  import TagEditor from '../TagEditor.svelte';
  import { snapsAPI } from '#lib/api/snaps.ts';
  import { clip } from '#lib/stores/clip.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { snaps } from '#lib/stores/snaps.svelte.ts';
  import { plural } from '#lib/util/plural.ts';

  /** The stack the board shows: what is cut came from there ('' for the board). */
  let { open }: { open: string | null } = $props();

  let tagging = $state(false);
  let adding = $state<string[]>([]);
  let deleting = $state(false);

  const ids = $derived(clip.selecting?.ids ?? []);
  const count = $derived(plural(ids.length, 'snap'));

  /** Each selected snap keeps its tags and gains the new ones. */
  async function addTags() {
    for (const id of ids) {
      const tags = snaps.snaps.find((s) => s.id === id)?.tags ?? [];
      const merged = [...new Set([...tags, ...adding])];
      await snapsAPI.setTags({ id, tags: merged });
    }
    await snaps.load();
    tagging = false;
    clip.stop();
  }

  async function remove() {
    for (const id of ids) await snapsAPI.remove(id);
    await snaps.load();
    deleting = false;
    clip.stop();
  }

  function askDelete() {
    if (settings.asksDelete('snap')) deleting = true;
    else remove();
  }

  function startTagging() {
    adding = [];
    tagging = true;
  }
</script>

<ActionBar oncut={() => clip.cutSelected(open ?? '')} middle={{ label: 'Tag', icon: 'tag', onclick: startTagging }} ondelete={askDelete} />

{#if tagging}
  <Sheet title="Tag {count}" action="Add" disabled={adding.length === 0} onaction={addTags} onclose={() => (tagging = false)}>
    <TagEditor tags={adding} onchange={(next) => (adding = next)} />
  </Sheet>
{/if}

{#if deleting}
  <ActionSheet
    title="Delete {count}?"
    body="They go from the board, on every device after the next sync. You can turn this question off in Settings › General."
    confirm="Delete {count}"
    danger
    onconfirm={remove}
    onclose={() => (deleting = false)} />
{/if}
