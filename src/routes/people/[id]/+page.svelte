<script lang="ts">
  // Person page: name fields, birthday, contact sections, notes and assigned tasks.
  import { page } from '$app/state';
  import Icon from '#lib/components/Icon.svelte';
  import Breadcrumbs from '#lib/components/Breadcrumbs.svelte';
  import PageBar from '#lib/components/PageBar.svelte';
  import Section from '#lib/components/Section.svelte';
  import ContactSection from '#lib/components/ContactSection.svelte';
  import MarkdownField from '#lib/components/MarkdownField.svelte';
  import EditControls from '#lib/components/EditControls.svelte';
  import DeleteDialog from '#lib/components/DeleteDialog.svelte';
  import PersonFields from '#lib/components/PersonFields.svelte';
  import PersonTasks from '#lib/components/PersonTasks.svelte';
  import { peopleAPI } from '#lib/api/people.ts';
  import { displayName } from '#lib/util/people.ts';
  import type { ContactKind, Person, PersonPatch } from '#lib/types/person.type.ts';
  import { people } from '#lib/stores/people.svelte.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { routes } from '#lib/util/routes.ts';
  import { nav } from '#lib/stores/history.svelte.ts';

  const CONTACT_SECTIONS: { kind: ContactKind; title: string; placeholder: string }[] = [
    { kind: 'phone', title: 'Phone', placeholder: '+91 98765 43210' },
    { kind: 'email', title: 'Email', placeholder: 'name@example.com' },
    { kind: 'address', title: 'Address', placeholder: 'street, city, postcode' },
  ];

  tasks.load();

  const id = $derived(page.params.id!);
  const isNew = $derived(page.url.searchParams.get('new') === '1');
  let person = $state<Person | null>(null);

  let notesField = $state<MarkdownField>();
  let notesEditing = $state(false);
  let deleting = $state(false);

  async function load() {
    person = await peopleAPI.get(id);
  }

  $effect(() => {
    void id;
    load();
  });

  async function save(patch: PersonPatch) {
    person = await peopleAPI.update({ id, patch });
    people.load();
  }

  async function remove() {
    deleting = false;
    const name = person ? displayName(person) : '';
    await peopleAPI.remove(id);
    await Promise.all([people.load(), tasks.load()]);
    toast.show(`Deleted ${name}`);
    nav.back(routes.people);
  }
</script>

<main class="main">
  {#if person}
    <div class="page">
      <PageBar onback={() => nav.back(routes.people)}>
        <Breadcrumbs crumbs={[{ label: 'People', href: routes.people }, { label: displayName(person) }]} />
        {#snippet actions()}
          <button class="icon-btn sm danger" aria-label="Delete person" title="Delete" onclick={() => (deleting = true)}><Icon name="trash" size={13} /></button>
        {/snippet}
      </PageBar>

      <div class="body">
        <h1 class="heading">{displayName(person)}</h1>
        <PersonFields {person} onsave={save} focusName={isNew} />

        <div class="sections">
          {#each CONTACT_SECTIONS as c (c.kind)}
            <ContactSection person={id} kind={c.kind} title={c.title} placeholder={c.placeholder} contacts={person.contacts} onchange={load} />
          {/each}
          <Section title="Notes">
            {#snippet actions()}
              <EditControls editing={notesEditing} what="notes" onedit={() => notesField?.edit()} oncancel={() => notesField?.cancel()} onsave={() => notesField?.save()} />
            {/snippet}
            <MarkdownField
              bind:this={notesField}
              bind:editing={notesEditing}
              value={person.notes}
              onsave={(notes) => save({ notes })}
              placeholder="Anything worth remembering (markdown)"
              empty="No notes"
              label="Notes" />
          </Section>
          <PersonTasks person={id} />
        </div>
      </div>
    </div>
  {/if}
</main>

{#if deleting && person}
  <DeleteDialog
    title="Delete {displayName(person)}?"
    body="Their contacts and notes are deleted, and their tasks become unassigned."
    ondelete={remove}
    onclose={() => (deleting = false)} />
{/if}

<style>
  .main {
    flex: 1 1 auto;
    min-width: 0;
    padding: 0 24px 72px;
    overflow: auto;
  }

  .page {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }


  .body {
    width: 100%;
    max-width: var(--content-w);
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 28px;
  }

  .sections {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
</style>
