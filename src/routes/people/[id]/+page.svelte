<script lang="ts">
  // Person page: name fields, birthday, contact sections, notes and assigned tasks.
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import Icon from '#lib/components/Icon.svelte';
  import Breadcrumbs from '#lib/components/Breadcrumbs.svelte';
  import Section from '#lib/components/Section.svelte';
  import ContactSection from '#lib/components/ContactSection.svelte';
  import MarkdownField from '#lib/components/MarkdownField.svelte';
  import EditControls from '#lib/components/EditControls.svelte';
  import StatusDot from '#lib/components/StatusDot.svelte';
  import DeleteDialog from '#lib/components/DeleteDialog.svelte';
  import { displayName, peopleApi, type Person, type PersonPatch } from '#lib/api/people.ts';
  import { messageOf } from '#lib/api/core.ts';
  import { people } from '#lib/stores/people.svelte.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { formatDob, parseDob, parseTime, timeLabel } from '#lib/util/dates.ts';
  import { birthdayHint, dobOf } from '#lib/util/birthdays.ts';
  import { isDone } from '#lib/util/task-views.ts';
  import { routes } from '#lib/util/routes.ts';

  tasks.load();

  const id = $derived(page.params.id!);
  let person = $state<Person | null>(null);

  let first = $state('');
  let last = $state('');
  let nick = $state('');
  let dob = $state('');
  let tob = $state('');

  function fill(p: Person) {
    person = p;
    first = p.firstName;
    last = p.lastName;
    nick = p.nickname ?? '';
    const d = dobOf(p);
    dob = d ? formatDob(d) : '';
    tob = p.tob ? timeLabel(p.tob) : '';
  }

  async function load() {
    try {
      fill(await peopleApi.get(id));
    } catch (e) {
      toast.show(messageOf(e));
    }
  }

  $effect(() => {
    void id;
    load();
  });

  async function save(patch: PersonPatch) {
    try {
      fill(await peopleApi.update(id, patch));
      people.load();
    } catch (e) {
      toast.show(messageOf(e));
    }
  }

  // ---- fields (saved on blur or Enter)

  const parsedDob = $derived(parseDob(dob));
  const dobBad = $derived(parsedDob === 'invalid');
  const parsedTob = $derived(tob.trim() ? parseTime(tob) : null);
  const tobBad = $derived(tob.trim() !== '' && parsedTob === null);

  function saveName() {
    if (!person) return;
    if (!first.trim()) {
      first = person.firstName;
      return;
    }
    const patch: PersonPatch = {};
    if (first.trim() !== person.firstName) patch.firstName = first;
    if (last.trim() !== person.lastName) patch.lastName = last;
    if ((nick.trim() || null) !== person.nickname) patch.nickname = nick.trim() || null;
    if (Object.keys(patch).length) save(patch);
  }

  function saveDob() {
    if (!person || dobBad) return;
    const d = parsedDob === 'invalid' ? null : parsedDob;
    const cur = dobOf(person);
    if ((d ? formatDob(d) : '') !== (cur ? formatDob(cur) : '')) save({ dob: d });
  }

  function saveTob() {
    if (!person || tobBad) return;
    if (parsedTob !== person.tob) save({ tob: parsedTob });
    else tob = person.tob ? timeLabel(person.tob) : '';
  }

  const enterBlurs = (e: KeyboardEvent) => e.key === 'Enter' && (e.currentTarget as HTMLInputElement).blur();

  const hint = $derived.by(() => {
    if (dobBad) return 'use yyyy-mm-dd or mm-dd';
    return parsedDob && parsedDob !== 'invalid' ? birthdayHint(parsedDob) : '';
  });

  // ---- notes, tasks, delete

  let notesField = $state<MarkdownField>();
  let notesEditing = $state(false);
  const assigned = $derived(tasks.tasks.filter((t) => t.assigneeId === id).sort((a, b) => Number(isDone(a)) - Number(isDone(b))));

  let deleting = $state(false);

  async function remove() {
    deleting = false;
    await peopleApi.remove(id);
    await Promise.all([people.load(), tasks.load()]);
    toast.show(`Deleted ${person ? displayName(person) : ''}`);
    goto('/people', { replace: true });
  }
</script>

<main class="main">
  {#if person}
    <div class="page">
      <div class="bar caption">
        <button class="icon-btn sm back" aria-label="Back to people" title="Back" onclick={() => goto('/people')}><Icon name="back" size={13} /></button>
        <Breadcrumbs crumbs={[{ label: 'People', href: '/people' }, { label: displayName(person) }]} />
        <span class="grow"></span>
        <button class="icon-btn sm danger" aria-label="Delete person" title="Delete" onclick={() => (deleting = true)}><Icon name="trash" size={13} /></button>
      </div>

      <div class="body">
        <h1 class="heading">{displayName(person)}</h1>
        <div class="props">
          <span class="pl">First name</span>
          <div class="pv"><input class="fld" style:width="240px" aria-label="First name" bind:value={first} onblur={saveName} onkeydown={enterBlurs} /></div>
          <span class="pl">Last name</span>
          <div class="pv"><input class="fld" style:width="240px" aria-label="Last name" bind:value={last} onblur={saveName} onkeydown={enterBlurs} /></div>
          <span class="pl">Nickname</span>
          <div class="pv">
            <input class="fld" style:width="240px" aria-label="Nickname" placeholder="optional, shown instead of the name" bind:value={nick} onblur={saveName} onkeydown={enterBlurs} />
          </div>
          <span class="pl">Date of birth</span>
          <div class="pv">
            <input class="fld mono" class:bad={dobBad} style:width="112px" aria-label="Date of birth, yyyy-mm-dd or mm-dd" placeholder="yyyy-mm-dd" bind:value={dob} onblur={saveDob} onkeydown={enterBlurs} />
            <span class="hint" class:err={dobBad}>{hint}</span>
          </div>
          <span class="pl">Time of birth</span>
          <div class="pv">
            <input class="fld mono" class:bad={tobBad} style:width="92px" aria-label="Time of birth" placeholder="optional" bind:value={tob} onblur={saveTob} onkeydown={enterBlurs} />
            {#if tobBad}<span class="hint err">use a time like 6:30 AM</span>{/if}
          </div>
        </div>

        <div class="sections">
          <ContactSection person={id} kind="phone" title="Phone" placeholder="+91 98765 43210" contacts={person.contacts} onchange={load} />
          <ContactSection person={id} kind="email" title="Email" placeholder="name@example.com" contacts={person.contacts} onchange={load} />
          <ContactSection person={id} kind="address" title="Address" placeholder="street, city, postcode" contacts={person.contacts} onchange={load} />
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
          <Section title="Assigned tasks">
            {#each assigned as t (t.id)}
              <a class="trow" href={routes.task(t.id)}>
                <StatusDot value={t.status} />
                <span class:done={isDone(t)}>{t.title}</span>
              </a>
            {/each}
            {#if assigned.length === 0}<span class="caption muted none">No tasks assigned</span>{/if}
          </Section>
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
    padding: 40px 24px 72px;
    overflow: auto;
  }

  .page {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .bar {
    display: flex;
    align-items: center;
    gap: 6px;
    min-height: 28px;
  }

  .back {
    margin-left: -6px;
  }

  .grow {
    flex: 1;
  }

  .body {
    width: 100%;
    max-width: var(--content-w);
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 28px;
  }

  .props {
    display: grid;
    grid-template-columns: 120px minmax(0, 1fr);
    row-gap: 6px;
    align-items: center;
  }

  .pl {
    font: 400 13px/18px var(--font-ui);
    color: var(--c-muted-2);
  }

  .pv {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 32px;
    font: 400 14px/20px var(--font-ui);
  }

  .fld {
    height: 28px;
    padding: 0 8px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    outline: none;
    background: var(--c-page);
    font: 400 14px/20px var(--font-ui);
  }

  .fld:focus {
    border-color: var(--primary);
  }

  .fld.mono {
    font: 400 13px/18px var(--font-mono);
  }

  .fld.bad {
    border-color: var(--danger);
    color: var(--danger);
  }

  .hint {
    font: 400 12px/16px var(--font-ui);
    color: var(--c-muted-2);
  }

  .hint.err {
    color: var(--danger);
  }

  .sections {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .trow {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 32px;
    padding: 0 16px;
    margin: 0 -16px;
    font: 400 14px/20px var(--font-ui);
    color: var(--text);
    text-decoration: none;
  }

  .trow:hover {
    background: var(--c-hover);
    color: var(--text);
  }

  .done {
    color: var(--c-muted-2);
  }

  .none {
    padding: 6px 0;
  }
</style>
