<script lang="ts">
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { focusWhen } from '#lib/actions/focus.action.ts';
  // Person page name and birth fields. A name is saved on blur or Enter; the date and time of birth
  // are the date and time fields a task's due date uses, side by side, saved as they change. Only
  // changed fields are sent.
  import type { Person, PersonPatch } from '#lib/types/person.type.ts';
  import type { DOB } from '#lib/types/date.type.ts';
  import DateField from './DateField.svelte';
  import TimeField from './TimeField.svelte';
  import { dobLabel, formatDOB, parseDOB } from '#lib/util/dates.ts';
  import { birthdayHint, dobOf } from '#lib/util/birthdays.ts';

  let {
    person,
    onsave,
    focusName = false,
  }: {
    person: Person;
    onsave: (patch: PersonPatch) => void;
    /** Focus the first name on open (a person just created). */
    focusName?: boolean;
  } = $props();

  let first = $state('');
  let last = $state('');
  let nick = $state('');

  /** A date of birth as the date picker holds it: 'yyyy-mm-dd', or 'mm-dd' without a year. */
  const dobText = (d: DOB | null) => (d ? formatDOB(d) : null);
  /** The picker's text back to a date of birth; null for none (or text it can't read). */
  function dobFrom(day: string | null): DOB | null {
    const parsed = day ? parseDOB(day) : null;
    return parsed === 'invalid' ? null : parsed;
  }

  // Every save or reload hands in a new person; the fields start over from it.
  $effect.pre(() => {
    first = person.firstName;
    last = person.lastName;
    nick = person.nickname ?? '';
  });

  const dob = $derived(dobOf(person));

  const hint = $derived(dob ? birthdayHint(dob) : '');

  /** The name fields that differ from the saved person. */
  function namePatch(): PersonPatch {
    const patch: PersonPatch = {};
    const nickname = nick.trim() || null;
    if (first.trim() !== person.firstName) patch.firstName = first;
    if (last.trim() !== person.lastName) patch.lastName = last;
    if (nickname !== person.nickname) patch.nickname = nickname;
    return patch;
  }

  function saveName() {
    // A person always has a first name: clearing it puts the saved one back.
    if (!first.trim()) {
      first = person.firstName;
      return;
    }
    const patch = namePatch();
    if (Object.keys(patch).length) onsave(patch);
  }

  function saveDOB(day: string | null) {
    const changed = day !== dobText(dob);
    if (changed) onsave({ dob: dobFrom(day) });
  }

  /** The button's text for the day the picker holds. */
  function dobShown(day: string): string {
    const d = dobFrom(day);
    return d ? dobLabel(d) : day;
  }

  function enterBlurs(e: KeyboardEvent) {
    if (e.key !== 'Enter') return;
    (e.currentTarget as HTMLInputElement).blur();
  }
</script>

<div class="props" data-zone="props">
  <span class="pl">First name</span>
  <div class="pv" data-prop><input use:plainText class="text-field quiet" style:width="240px" aria-label="First name" use:focusWhen={focusName} bind:value={first} onblur={saveName} onkeydown={enterBlurs} /></div>
  <span class="pl">Last name</span>
  <div class="pv" data-prop><input use:plainText class="text-field quiet" style:width="240px" aria-label="Last name" bind:value={last} onblur={saveName} onkeydown={enterBlurs} /></div>
  <span class="pl">Nickname</span>
  <div class="pv" data-prop>
    <input use:plainText class="text-field quiet" style:width="240px" aria-label="Nickname" placeholder="optional, shown instead of the name" bind:value={nick} onblur={saveName} onkeydown={enterBlurs} />
  </div>
  <span class="pl">Born</span>
  <div class="pv" data-prop>
    <DateField value={dobText(dob)} what="date of birth" format={dobShown} dimPast={false} yearOptional onchange={saveDOB} />
    <TimeField value={person.tob} onchange={(tob) => onsave({ tob })} />
    <span class="hint">{hint}</span>
  </div>
</div>

<style>
  .hint {
    font: var(--text-meta);
    color: var(--c-muted-2);
  }
</style>
