<script lang="ts">
  // People list: search, add (first and last name, ↵ opens the new person), and the table.
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import Icon from '#lib/components/Icon.svelte';
  import { peopleAPI } from '#lib/api/people.ts';
  import { displayName, personMatches, splitName } from '#lib/util/people.ts';
  import type { PersonRow } from '#lib/types/person.type.ts';
  import { messageOf } from '#lib/api/core.ts';
  import { people } from '#lib/stores/people.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { personBirthday } from '#lib/util/birthdays.ts';
  import { plural } from '#lib/util/plural.ts';
  import { routes } from '#lib/util/routes.ts';

  const UPCOMING_DAYS = 30;

  people.load();

  let query = $state('');
  let draft = $state('');
  let addInput = $state<HTMLInputElement>();

  $effect(() => {
    if (page.url.searchParams.get('add') === '1') addInput?.focus();
  });

  const needle = $derived(query.trim().toLowerCase());
  const shown = $derived(people.people.filter((p) => personMatches(p, needle)));
  const noMatches = $derived(shown.length === 0 && people.people.length > 0);

  function hasUpcomingBirthday(p: PersonRow): boolean {
    const birthday = personBirthday(p);
    return birthday !== null && birthday.days <= UPCOMING_DAYS;
  }

  const upcoming = $derived(people.people.filter(hasUpcomingBirthday).length);

  async function add() {
    const name = splitName(draft);
    if (!name.first) return;
    try {
      const id = await peopleAPI.create(name);
      draft = '';
      await people.load();
      goto(routes.person(id));
    } catch (e) {
      toast.show(messageOf(e));
    }
  }

  function addKeys(e: KeyboardEvent) {
    if (e.key === 'Enter') add();
  }
</script>

<main class="main">
  <div class="page">
    <div class="head">
      <h1 class="heading">People</h1>
      <span class="caption muted">{plural(people.people.length, 'person', 'people')} · {plural(upcoming, 'birthday')} in the next {UPCOMING_DAYS} days</span>
    </div>
    <label class="search">
      <span class="ic"><Icon name="search" size={14} /></span>
      <input aria-label="Search people" placeholder="Search name, phone or email" bind:value={query} />
    </label>
    <div class="table">
      <label class="add">
        <span class="plus">+</span>
        <input bind:this={addInput} aria-label="Add person" placeholder="add person (first and last name), ↵" bind:value={draft} onkeydown={addKeys} />
      </label>
      <div class="phead"><span>Name</span><span>Birthday</span><span>Phone</span><span>Email</span></div>
      {#each shown as p (p.id)}
        {@const bd = personBirthday(p)}
        <a class="prow" href={routes.person(p.id)}>
          <span class="nm">{displayName(p)}</span>
          <span class="m" class:soon={bd?.soon}>{bd?.text ?? ''}</span>
          <span class="m">{p.phone ?? ''}</span>
          <span class="m">{p.email ?? ''}</span>
        </a>
      {/each}
      {#if noMatches}<span class="caption muted none">No one matches</span>{/if}
    </div>
  </div>
</main>

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
    gap: 20px;
  }

  .head {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 32px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: 6px;
    max-width: 320px;
  }

  .search:focus-within {
    border-color: var(--primary);
  }

  .search .ic {
    display: grid;
    color: var(--c-muted-2);
  }

  .search input {
    flex: 1;
    border: 0;
    outline: none;
    background: transparent;
    font: 400 14px/20px var(--font-ui);
  }

  .table {
    display: flex;
    flex-direction: column;
  }

  .add {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 36px;
    font: 400 14px/20px var(--font-ui);
    color: var(--c-muted-2);
  }

  .plus {
    width: 16px;
    text-align: center;
  }

  .add input {
    flex: 1;
    border: 0;
    outline: none;
    background: transparent;
    font: 400 14px/20px var(--font-ui);
    color: var(--text);
  }

  .add input::placeholder {
    color: var(--c-muted-2);
  }

  .phead,
  .prow {
    display: grid;
    grid-template-columns: minmax(0, 1.2fr) minmax(0, 1fr) minmax(0, 1fr) minmax(0, 1.2fr);
    gap: 16px;
  }

  .phead {
    padding: 0 0 6px;
    border-bottom: 1px solid var(--c-chip);
    font: 400 12px/16px var(--font-ui);
    color: var(--c-faint);
  }

  .prow {
    align-items: center;
    min-height: 40px;
    padding: 0 16px;
    margin: 0 -16px;
    font: 400 14px/20px var(--font-ui);
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
    font-size: 13px;
  }

  .m.soon {
    color: var(--accent);
  }

  .none {
    padding: 10px 0;
  }
</style>
