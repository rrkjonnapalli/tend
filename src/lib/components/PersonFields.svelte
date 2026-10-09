<script lang="ts">
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { focusWhen } from '#lib/actions/focus.action.ts';
  // Person page name and birth fields. Each is saved on blur or Enter; only changed fields are sent.
  import type { Person, PersonPatch } from '#lib/types/person.type.ts';
  import type { DOB } from '#lib/types/date.type.ts';
  import { formatDOB, parseDOB, parseTime, timeLabel } from '#lib/util/dates.ts';
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
  let dob = $state('');
  let tob = $state('');

  const dobText = (d: DOB | null) => (d ? formatDOB(d) : '');
  const tobText = (hhmm: string | null) => (hhmm ? timeLabel(hhmm) : '');

  // Every save or reload hands in a new person; the fields start over from it.
  $effect.pre(() => {
    first = person.firstName;
    last = person.lastName;
    nick = person.nickname ?? '';
    dob = dobText(dobOf(person));
    tob = tobText(person.tob);
  });

  const parsedDOB = $derived(parseDOB(dob));
  const dobBad = $derived(parsedDOB === 'invalid');
  const hasTOB = $derived(tob.trim() !== '');
  const parsedTOB = $derived(hasTOB ? parseTime(tob) : null);
  const tobBad = $derived(hasTOB && parsedTOB === null);

  const hint = $derived.by(() => {
    if (parsedDOB === 'invalid') return 'use yyyy-mm-dd or mm-dd';
    if (parsedDOB) return birthdayHint(parsedDOB);
    return '';
  });

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

  function saveDOB() {
    if (parsedDOB === 'invalid') return;
    const changed = dobText(parsedDOB) !== dobText(dobOf(person));
    if (changed) onsave({ dob: parsedDOB });
  }

  function saveTOB() {
    if (tobBad) return;
    if (parsedTOB !== person.tob) onsave({ tob: parsedTOB });
    else tob = tobText(person.tob);
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
  <span class="pl">Date of birth</span>
  <div class="pv" data-prop>
    <input use:plainText class="text-field mono quiet" class:bad={dobBad} style:width="112px" aria-label="Date of birth, yyyy-mm-dd or mm-dd" placeholder="yyyy-mm-dd" bind:value={dob} onblur={saveDOB} onkeydown={enterBlurs} />
    <span class="hint" class:err={dobBad}>{hint}</span>
  </div>
  <span class="pl">Time of birth</span>
  <div class="pv" data-prop>
    <input use:plainText class="text-field mono quiet" class:bad={tobBad} style:width="92px" aria-label="Time of birth" placeholder="optional" bind:value={tob} onblur={saveTOB} onkeydown={enterBlurs} />
    {#if tobBad}<span class="hint err">use a time like 6:30 AM</span>{/if}
  </div>
</div>

<style>
  .hint {
    font: var(--text-meta);
    color: var(--c-muted-2);
  }

  .hint.err {
    color: var(--danger);
  }
</style>
