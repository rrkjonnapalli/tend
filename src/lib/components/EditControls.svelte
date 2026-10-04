<script lang="ts">
  // Pencil to start editing; red ✕ and green ✓ while editing.
  import Icon from './Icon.svelte';

  let {
    editing,
    onedit,
    oncancel,
    onsave,
    what = '',
    hover = false,
  }: {
    editing: boolean;
    onedit: () => void;
    oncancel: () => void;
    onsave: () => void;
    /** Names the thing for screen readers: "Edit description". */
    what?: string;
    /** Show the pencil only while a `.reveal-on-hover` ancestor is hovered. */
    hover?: boolean;
  } = $props();
</script>

<span class="acts">
  {#if editing}
    <button class="icon-btn sm danger" aria-label="Cancel" title="Cancel" onclick={oncancel}><Icon name="close" size={13} stroke={2.25} /></button>
    <button class="icon-btn sm ok" aria-label="Save" title="Save" onclick={onsave}><Icon name="check" size={13} stroke={2.25} /></button>
  {:else}
    <button class="icon-btn sm edit" class:on-hover={hover} aria-label={what ? `Edit ${what}` : 'Edit'} title="Edit" onclick={onedit}>
      <Icon name="edit" size={13} />
    </button>
  {/if}
</span>

<style>
  .acts {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .edit {
    color: var(--c-muted-2);
  }

  .edit:hover {
    background: var(--border);
  }
</style>
