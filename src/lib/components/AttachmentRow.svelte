<script lang="ts">
  // One attached file or linked note: icon, name, meta, and open / show in folder / remove on hover.
  import { goto } from '$app/navigation';
  import Icon from './Icon.svelte';
  import { attachmentsAPI } from '#lib/api/attachments.ts';
  import type { AttachmentLink } from '#lib/types/attachment.type.ts';
  import { attachmentIcon, attachmentMeta } from '#lib/util/attachments.ts';
  import { routes } from '#lib/util/routes.ts';

  let { link, onremove }: { link: AttachmentLink; onremove: () => void } = $props();

  async function openLink() {
    if (link.kind === 'note' && link.noteId) {
      goto(routes.note(link.noteId));
      return;
    }
    if (!link.attachmentId) return;
    await attachmentsAPI.open(link.attachmentId);
  }

</script>

<div class="arow reveal-on-hover">
  <span class="ic" class:note={link.kind === 'note'}>
    <Icon name={attachmentIcon(link)} size={14} />
  </span>
  <span class="an">{link.name}</span>
  <span class="am">{attachmentMeta(link)}</span>
  <span class="acts on-hover">
    <button class="aib" aria-label="Open" title="Open" onclick={openLink}><Icon name="open" size={14} /></button>
    {#if link.kind === 'file' && link.attachmentId}
      {@const attachmentId = link.attachmentId}
      <button class="aib" aria-label="Show in folder" title="Show in folder" onclick={() => attachmentsAPI.reveal(attachmentId)}>
        <Icon name="folder" size={14} />
      </button>
    {/if}
    <button class="aib del" aria-label="Remove" title="Remove" onclick={onremove}><Icon name="close" size={12} stroke={2.25} /></button>
  </span>
</div>

<style>
  .ic {
    display: grid;
    color: var(--c-muted-2);
  }

  .ic.note {
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
    font: var(--text-ui);
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
    font: var(--text-meta);
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
