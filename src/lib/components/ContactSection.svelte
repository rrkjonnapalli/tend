<script lang="ts">
  // Phone / Email / Address rows: value + label, edit or delete each, and an add row at the end.
  import Icon from './Icon.svelte';
  import Section from './Section.svelte';
  import { peopleAPI } from '#lib/api/people.ts';
  import type { Contact, ContactKind } from '#lib/types/person.type.ts';
  import { messageOf } from '#lib/api/core.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';

  let {
    person,
    kind,
    title,
    placeholder,
    contacts,
    onchange,
  }: {
    person: string;
    kind: ContactKind;
    title: string;
    placeholder: string;
    contacts: Contact[];
    onchange: () => void;
  } = $props();

  const rows = $derived(contacts.filter((c) => c.kind === kind));
  let editing = $state<string | null>(null);
  let edValue = $state('');
  let edLabel = $state('');
  let newValue = $state('');
  let newLabel = $state('');

  async function run(action: () => Promise<void>) {
    try {
      await action();
      onchange();
    } catch (e) {
      toast.show(messageOf(e));
    }
  }

  function edit(c: Contact) {
    editing = c.id;
    edValue = c.value;
    edLabel = c.label;
  }

  const save = () =>
    run(async () => {
      if (!editing || !edValue.trim()) return;
      await peopleAPI.updateContact({ id: editing, value: edValue, label: edLabel });
      editing = null;
    });

  const add = () =>
    run(async () => {
      if (!newValue.trim()) return;
      await peopleAPI.addContact({ person, kind, value: newValue, label: newLabel });
      newValue = '';
      newLabel = '';
    });

  function keys(e: KeyboardEvent, onEnter: () => void, onEscape: () => void) {
    if (e.key === 'Enter') onEnter();
    if (e.key === 'Escape') {
      e.stopPropagation();
      onEscape();
    }
  }

  function focusOnMount(node: HTMLInputElement) {
    node.focus();
  }
</script>

<Section {title}>
  {#each rows as c (c.id)}
    {@const isEditing = editing === c.id}
    <div class="crow reveal-on-hover" class:editing={isEditing}>
      {#if isEditing}
        <input class="ein" aria-label="{title} value" bind:value={edValue} use:focusOnMount onkeydown={(e) => keys(e, save, () => (editing = null))} />
        <input class="ein lab" aria-label="Label" placeholder="label" bind:value={edLabel} onkeydown={(e) => keys(e, save, () => (editing = null))} />
      {:else}
        <span class="cv">{c.value}</span>
        <span class="cl">{c.label}</span>
      {/if}
      <span class="acts" class:on-hover={!isEditing}>
        <button class="icon-btn sm danger" aria-label="Delete" title="Delete" onclick={() => run(() => peopleAPI.removeContact(c.id))}>
          <Icon name="trash" size={13} />
        </button>
        {#if isEditing}
          <button class="icon-btn sm ok" aria-label="Save" title="Save" onclick={save}><Icon name="check" size={13} stroke={2.25} /></button>
          <button class="icon-btn sm danger" aria-label="Cancel" title="Cancel" onclick={() => (editing = null)}><Icon name="close" size={13} stroke={2.25} /></button>
        {:else}
          <button class="icon-btn sm edit" aria-label="Edit" title="Edit" onclick={() => edit(c)}><Icon name="edit" size={13} /></button>
        {/if}
      </span>
    </div>
  {/each}
  <div class="crow add">
    <span class="plus">+</span>
    <input class="ain" aria-label="New {title}" {placeholder} bind:value={newValue} onkeydown={(e) => keys(e, add, () => ((newValue = ''), (newLabel = '')))} />
    <input class="ain lab" aria-label="Label" placeholder="label" bind:value={newLabel} onkeydown={(e) => keys(e, add, () => ((newValue = ''), (newLabel = '')))} />
    <span class="acts">
      {#if newValue.trim()}<button class="icon-btn sm ok" aria-label="Add" title="Add" onclick={add}><Icon name="check" size={13} stroke={2.25} /></button>{/if}
    </span>
  </div>
</Section>

<style>
  .crow {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 140px 84px;
    align-items: center;
    gap: 12px;
    min-height: 36px;
    padding: 0 16px;
    margin: 0 -16px;
    font: 400 14px/20px var(--font-ui);
  }

  .crow:hover {
    background: var(--c-hover);
  }

  .crow.editing,
  .crow.add {
    background: transparent;
  }

  .crow.add {
    grid-template-columns: 16px minmax(0, 1fr) 140px 84px;
    color: var(--c-muted-2);
  }

  .plus {
    text-align: center;
  }

  .cv {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cl {
    font: 400 12px/16px var(--font-ui);
    color: var(--c-muted-2);
  }

  .acts {
    display: flex;
    justify-content: flex-end;
    gap: 2px;
  }

  .edit {
    color: var(--primary);
  }

  .edit:hover {
    background: var(--primary-soft);
    color: var(--primary);
  }

  .ein {
    min-width: 0;
    height: 28px;
    padding: 0 8px;
    margin-left: -8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: var(--c-hover);
    outline: none;
    font: 400 14px/20px var(--font-ui);
  }

  .ein.lab,
  .ain.lab {
    font-size: 13px;
    margin-left: 0;
  }

  .ain {
    min-width: 0;
    border: 0;
    outline: none;
    background: transparent;
    font: 400 14px/20px var(--font-ui);
    color: var(--text);
  }

  .ain::placeholder {
    color: var(--c-faint);
  }
</style>
