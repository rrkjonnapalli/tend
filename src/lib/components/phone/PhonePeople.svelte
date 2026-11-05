<script lang="ts">
  // People on a phone (MD-People): search and +, the birthdays coming up in a collapsible section,
  // then everyone by name. + asks for a first and last name and opens the new person.
  import { goto } from '$app/navigation';
  import Icon from '../Icon.svelte';
  import PhoneSearch from './PhoneSearch.svelte';
  import Sheet from './Sheet.svelte';
  import { focusOnMount } from '#lib/actions/focus.action.ts';
  import { peopleAPI } from '#lib/api/people.ts';
  import { people } from '#lib/stores/people.svelte.ts';
  import { displayName, personMatches } from '#lib/util/people.ts';
  import { personBirthday } from '#lib/util/birthdays.ts';
  import { searchNeedle } from '#lib/util/search.ts';
  import { routes } from '#lib/util/routes.ts';

  const UPCOMING_DAYS = 30;

  let query = $state('');
  let open = $state(true);
  let adding = $state(false);
  let first = $state('');
  let last = $state('');

  const needle = $derived(searchNeedle(query));
  const shown = $derived(people.people.filter((p) => personMatches(p, needle)));
  const soon = $derived.by(() => {
    const rows = people.people.flatMap((p) => {
      const bd = personBirthday(p);
      return bd && bd.days <= UPCOMING_DAYS ? [{ p, bd }] : [];
    });
    return rows.sort((a, b) => a.bd.days - b.bd.days);
  });

  /** "24 Oct · in 5 days": the list label without the age. */
  const dateOf = (text: string) => text.split(' · ').slice(0, 2).join(' · ');

  async function add() {
    const id = await peopleAPI.create({ first: first.trim(), last: last.trim() });
    await people.load();
    adding = false;
    goto(routes.person(id, { isNew: true }));
  }

</script>

<main class="phone-list">
  <PhoneSearch placeholder="Search name, phone or email" bind:query onadd={() => (adding = true)} />

  {#if !needle && soon.length > 0}
    <button class="fs" aria-expanded={open} onclick={() => (open = !open)}>
      <Icon name={open ? 'chevronDown' : 'chevronRight'} size={14} stroke={2.5} />Birthdays<span class="n">{soon.length}</span>
    </button>
    {#if open}
      {#each soon as { p, bd } (p.id)}
        <a class="qr in" href={routes.person(p.id)}><span class="tt">{displayName(p)}</span><span class="bd">{dateOf(bd.text)}</span></a>
      {/each}
    {/if}
    <div class="gap"></div>
  {/if}

  {#each shown as p (p.id)}
    <a class="qr" href={routes.person(p.id)}><span class="tt">{displayName(p)}</span></a>
  {:else}
    <p class="empty">{people.people.length > 0 ? 'No one matches' : 'No people yet'}</p>
  {/each}
</main>

{#if adding}
  <Sheet title="New person" action="Add" disabled={first.trim() === ''} onaction={add} onclose={() => (adding = false)}>
    <label class="f"><span class="lbl">First name</span><input use:focusOnMount class="in" autocapitalize="words" bind:value={first} /></label>
    <label class="f"><span class="lbl">Last name</span><input class="in" autocapitalize="words" bind:value={last} /></label>
  </Sheet>
{/if}

<style>
  .fs {
    display: flex;
    align-items: center;
    gap: 6px;
    width: calc(100% + 20px);
    height: 44px;
    margin: 0 -10px;
    padding: 0 10px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    font: var(--text-section);
    color: var(--text);
  }

  .fs :global(svg),
  .n {
    color: var(--c-muted-2);
  }

  .n {
    font: var(--text-meta);
  }

  .in {
    padding-left: 34px;
  }

  .bd {
    flex: none;
    font: var(--text-caption);
    color: var(--accent);
  }

  .gap {
    height: 12px;
  }

  .f {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .lbl {
    font: var(--text-caption);
    color: var(--text-muted);
  }

  .f .in {
    height: 44px;
    padding: 0 12px;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    background: var(--c-chip);
    font: var(--text-ui);
    font-size: 15px;
    color: var(--text);
  }

  .f .in:focus {
    outline: none;
    border-color: var(--primary);
  }
</style>
