<script lang="ts">
  // People list: search, add (first and last name, ↵ opens the new person), and the table.
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import Breadcrumbs from '#lib/components/Breadcrumbs.svelte';
  import PageBar from '#lib/components/PageBar.svelte';
  import PageTitle from '#lib/components/PageTitle.svelte';
  import ColumnHeader from '#lib/components/ColumnHeader.svelte';
  import SearchLine from '#lib/components/SearchLine.svelte';
  import AddLine from '#lib/components/AddLine.svelte';
  import { zoneNav } from '#lib/actions/zone-nav.action.ts';
  import { peopleAPI } from '#lib/api/people.ts';
  import { displayName, PEOPLE_ROOT_NAME, personMatches, splitName } from '#lib/util/people.ts';
  import type { PersonRow } from '#lib/types/person.type.ts';
  import { people } from '#lib/stores/people.svelte.ts';
  import { personBirthday } from '#lib/util/birthdays.ts';
  import { plural } from '#lib/util/plural.ts';
  import { routes } from '#lib/util/routes.ts';
  import { searchNeedle } from '#lib/util/search.ts';

  const UPCOMING_DAYS = 30;
  const COLUMNS = ['Name', 'Birthday', 'Phone', 'Email'];

  people.load();

  let query = $state('');
  let draft = $state('');
  let addInput = $state<HTMLInputElement>();

  $effect(() => {
    if (page.url.searchParams.get('add') === '1') addInput?.focus();
  });

  const needle = $derived(searchNeedle(query));
  const shown = $derived(people.people.filter((p) => personMatches(p, needle)));
  const emptyText = $derived(people.people.length > 0 ? 'No one matches' : 'No people yet');

  function hasUpcomingBirthday(p: PersonRow): boolean {
    const birthday = personBirthday(p);
    return birthday !== null && birthday.days <= UPCOMING_DAYS;
  }

  const upcoming = $derived(people.people.filter(hasUpcomingBirthday).length);
  const summary = $derived(`${plural(people.people.length, 'person', 'people')} · ${plural(upcoming, 'birthday')} in the next ${UPCOMING_DAYS} days`);

  async function add() {
    const name = splitName(draft);
    if (!name.first) return;
    const id = await peopleAPI.create(name);
    draft = '';
    await people.load();
    goto(routes.person(id, { isNew: true }));
  }
</script>

<main class="list-page" use:zoneNav>
  <PageBar><Breadcrumbs crumbs={[{ label: PEOPLE_ROOT_NAME }]} /></PageBar>
  <PageTitle title={PEOPLE_ROOT_NAME} meta={summary} />
  <SearchLine bind:value={query} label="Search people" placeholder="Search name, phone or email" />
  <AddLine bind:value={draft} bind:input={addInput} label="Add person" placeholder="add person (first and last name)" onadd={add} />
  <ColumnHeader template="var(--grid-person)" gap="16px" columns={COLUMNS} />
  <div class="list-scroll" data-zone="rows">
    <div class="table">
      {#each shown as p (p.id)}
        {@const bd = personBirthday(p)}
        <a class="prow" data-row href={routes.person(p.id)}>
          <span class="nm">{displayName(p)}</span>
          <span class="m" class:soon={bd?.soon}>{bd?.text ?? ''}</span>
          <span class="m">{p.phone ?? ''}</span>
          <span class="m">{p.email ?? ''}</span>
        </a>
      {/each}
      {#if people.loaded && shown.length === 0}<span class="list-empty">{emptyText}</span>{/if}
    </div>
  </div>
</main>

<style>

  .table {
    display: flex;
    flex-direction: column;
  }

  .prow {
    display: grid;
    grid-template-columns: var(--grid-person);
    gap: 16px;
  }

  .prow {
    align-items: center;
    min-height: 36px;
    padding: 0 16px;
    margin: 0 -16px;
    font: var(--text-ui);
    color: var(--text);
    text-decoration: none;
  }

  .prow:hover {
    background: var(--c-hover);
    color: var(--text);
  }

  .nm,
  .m {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .m {
    color: var(--c-muted-2);
    font: var(--text-caption);
  }

  .m.soon {
    color: var(--accent);
  }
</style>
