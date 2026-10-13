<script lang="ts">
  // Settings → Tags: every tag in two columns, Active and Blocked, each with how many things use
  // it. Blocking asks first: the tag comes off everything and can't be added until unblocked.
  import ConfirmDialog from './ConfirmDialog.svelte';
  import Section from './Section.svelte';
  import { tagsAPI } from '#lib/api/tags.ts';
  import { ENUM_STATUS, type EnumStatus } from '#lib/enums/status.enum.ts';
  import { reloadAll } from '#lib/stores/sync.svelte.ts';
  import { tagPool } from '#lib/stores/tags.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import type { TagRow } from '#lib/types/tag.type.ts';
  import { plural } from '#lib/util/plural.ts';

  tagPool.load();

  const active = $derived(tagPool.rows.filter((t) => t.status === ENUM_STATUS.active));
  const blocked = $derived(tagPool.rows.filter((t) => t.status === ENUM_STATUS.blocked));

  /** The tag whose block is being confirmed. */
  let asking = $state<TagRow | null>(null);

  /** Writes the status, then reloads the pool (pickers stop offering a blocked tag) and every
   *  list that shows tags. */
  async function setStatus(tag: TagRow, status: EnumStatus) {
    await tagsAPI.setStatus({ name: tag.name, status });
    await reloadAll();
  }

  async function block() {
    if (!asking) return;
    const tag = asking;
    await setStatus(tag, ENUM_STATUS.blocked);
    asking = null;
    toast.show(`Blocked “${tag.name}”`);
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
      </div>
    {/each}
    {#if tags.length === 0}<span class="list-empty">{empty}</span>{/if}
  </Section>
{/snippet}

<span class="caption muted">
  Tags are shared by notes, tasks, words, snaps and references. A blocked tag comes off everything and can’t be added again until you unblock it.
</span>
<div class="cols">
  {@render column({ title: 'Active', tags: active, empty: 'No tags yet.', action: 'Block', onact: (tag) => (asking = tag) })}
  {@render column({ title: 'Blocked', tags: blocked, empty: 'No blocked tags.', action: 'Unblock', onact: unblock })}
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

<style>
  .cols {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    align-items: start;
    gap: 48px;
  }

  .trow {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
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
