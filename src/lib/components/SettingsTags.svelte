<script lang="ts">
  // Settings → Tags: every tag in two columns, Active and Blocked, each with how many things use
  // it. Blocking asks first: the tag comes off everything and can't be added until unblocked.
  // Deleting asks too: the tag leaves the pool and everything that carries it, and can be typed again.
  import ConfirmDialog from './ConfirmDialog.svelte';
  import Icon from './Icon.svelte';
  import SearchLine from './SearchLine.svelte';
  import Section from './Section.svelte';
  import { tagsAPI } from '#lib/api/tags.ts';
  import { ENUM_STATUS, type EnumStatus } from '#lib/enums/status.enum.ts';
  import { reloadAll } from '#lib/stores/sync.svelte.ts';
  import { tagPool } from '#lib/stores/tags.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import type { TagRow } from '#lib/types/tag.type.ts';
  import { plural } from '#lib/util/plural.ts';
  import { searchNeedle, textMatches } from '#lib/util/search.ts';

  tagPool.load();

  /** The search covers both columns. */
  let query = $state('');
  const needle = $derived(searchNeedle(query));
  const found = $derived(tagPool.rows.filter((t) => textMatches([t.name], needle)));
  const active = $derived(found.filter((t) => t.status === ENUM_STATUS.active));
  const blocked = $derived(found.filter((t) => t.status === ENUM_STATUS.blocked));
  const noMatch = 'No tags match.';

  /** The tag whose block is being confirmed. */
  let asking = $state<TagRow | null>(null);
  /** The tag whose delete is being confirmed. */
  let deleting = $state<TagRow | null>(null);

  /** Writes the status, then reloads the pool (pickers stop offering a blocked tag) and every
   *  list that shows tags. */
  async function setStatus(tag: TagRow, status: EnumStatus) {
    await tagsAPI.setStatus({ name: tag.name, status });
    await reloadAll();
  }

  /** What deleting takes with it; a blocked or unused tag is on nothing. */
  function deleteBody(tag: TagRow): string {
    if (tag.uses === 0) return 'It isn’t on anything. This can’t be undone.';
    return `It comes off ${plural(tag.uses, 'thing')} that carry it. This can’t be undone.`;
  }

  async function block() {
    if (!asking) return;
    const tag = asking;
    await setStatus(tag, ENUM_STATUS.blocked);
    asking = null;
    toast.show(`Blocked “${tag.name}”`);
  }

  async function remove() {
    if (!deleting) return;
    const tag = deleting;
    deleting = null;
    await tagsAPI.remove(tag.name);
    await reloadAll();
    toast.show(`Deleted “${tag.name}”`);
  }

  async function unblock(tag: TagRow) {
    await setStatus(tag, ENUM_STATUS.active);
    toast.show(`Unblocked “${tag.name}”`);
  }
</script>

{#snippet column({ title, tags, empty, action, onact }: { title: string; tags: TagRow[]; empty: string; action: string; onact: (tag: TagRow) => void })}
  <Section {title} extra={String(tags.length)} indent={false}>
    {#each tags as tag (tag.name)}
      <div class="trow">
        <span class="nm" title={tag.name}>{tag.name}</span>
        <span class="uses">{plural(tag.uses, 'use')}</span>
        <button class="btn-text" aria-label="{action} tag {tag.name}" onclick={() => onact(tag)}>{action}</button>
        <button class="icon-btn sm danger" aria-label="Delete tag {tag.name}" title="Delete" onclick={() => (deleting = tag)}>
          <Icon name="trash" size={13} />
        </button>
      </div>
    {/each}
    {#if tags.length === 0}<span class="list-empty">{empty}</span>{/if}
  </Section>
{/snippet}

<SearchLine bind:value={query} label="Search tags" placeholder="Search tags" clearable />
<span class="caption muted">
  Tags are shared by notes, tasks, words, snaps and references. A blocked tag comes off everything and can’t be added again until you unblock it.
</span>
<div class="cols">
  {@render column({ title: 'Active', tags: active, empty: needle ? noMatch : 'No tags yet.', action: 'Block', onact: (tag) => (asking = tag) })}
  {@render column({ title: 'Blocked', tags: blocked, empty: needle ? noMatch : 'No blocked tags.', action: 'Unblock', onact: unblock })}
</div>

{#if asking}
  <ConfirmDialog
    title="Block “{asking.name}”?"
    body="It comes off {plural(asking.uses, 'thing')} that carry it, and can’t be added again until you unblock it."
    confirm="Block"
    danger
    onconfirm={block}
    onclose={() => (asking = null)} />
{/if}

{#if deleting}
  <ConfirmDialog
    title="Delete “{deleting.name}”?"
    body={deleteBody(deleting)}
    confirm="Delete"
    danger
    onconfirm={remove}
    onclose={() => (deleting = null)} />
{/if}

<style>
  .cols {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    align-items: start;
    gap: 48px;
  }

  .trow {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto auto;
    align-items: center;
    gap: 12px;
    min-height: 36px;
    font: var(--text-ui);
  }

  .nm {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .uses {
    font: var(--text-meta);
    color: var(--c-muted-2);
  }
</style>
