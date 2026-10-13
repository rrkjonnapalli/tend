<script lang="ts">
  // One word's page, for adding and for editing. Only the title is needed. Adding: Enter in the
  // title looks it up, Save (⌘↵) adds it, Cancel (Esc) drops it. A saved word keeps each change as
  // it is made; a title left empty or taken goes back to what it was.
  import { untrack } from 'svelte';
  import PageBar from './PageBar.svelte';
  import Breadcrumbs from './Breadcrumbs.svelte';
  import Chip from './Chip.svelte';
  import Icon from './Icon.svelte';
  import KindPicker from './KindPicker.svelte';
  import TagEditor from './TagEditor.svelte';
  import WordDictionary from './WordDictionary.svelte';
  import WordNote from './WordNote.svelte';
  import Section from './Section.svelte';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { focusWhen } from '#lib/actions/focus.action.ts';
  import { wordsAPI } from '#lib/api/words.ts';
  import { dictionarySource, idiomPartOfSpeech } from '#lib/config/library.config.ts';
  import { WORD_KIND } from '#lib/enums/word.enum.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { words } from '#lib/stores/words.svelte.ts';
  import type { Dictionary, Word, WordPatch } from '#lib/types/word.type.ts';
  import { iso, shortDate } from '#lib/util/dates.ts';
  import { guessKind, savedTitle } from '#lib/util/words.ts';

  let {
    word,
    filled = '',
    onsaved,
    oncancel,
    onopen,
    ondeleted,
    ontyped,
  }: {
    /** The word to edit; none adds a new one. */
    word: Word | null;
    /** A new word's title, handed over from Home. */
    filled?: string;
    onsaved: (id: string) => void;
    /** Drops a new word (Cancel, Esc); a saved word has nothing to cancel. */
    oncancel?: () => void;
    onopen: (id: string) => void;
    ondeleted: () => void;
    /** A new word's title as it is typed. */
    ontyped?: (title: string) => void;
  } = $props();

  // The page is remounted for each word, so these start from the one it opened with.
  const start = untrack(() => word);
  const isNew = start === null;
  const handedTitle = untrack(() => filled);
  let title = $state(start?.title ?? handedTitle);
  let kind = $state<string>(start?.kind ?? guessKind(handedTitle));
  let kindSet = $state(!isNew);
  let dictionary = $state<Dictionary | null>(start?.dictionary ?? null);
  /** The dictionary answered for what the title says now, and had no entry. */
  let notFound = $state(false);
  let fetching = $state(false);
  let meaning = $state(start?.meaning ?? '');
  let notes = $state(start?.notes ?? '');
  let tags = $state<string[]>(start?.tags ?? []);
  let confirming = $state(false);

  const kindInfo = $derived(words.kindOf(kind));
  const looksUp = $derived(kindInfo?.lookup ?? false);
  const dupe = $derived(title.trim() ? words.withTitle({ title: savedTitle({ title, kind }), except: start?.id }) : undefined);

  const hint = $derived.by(() => {
    if (!looksUp) return 'No dictionary for this one.';
    if (notFound) return 'Not in the dictionary.';
    // A saved word without an entry may never have been looked up (the dictionary was down).
    if (!isNew) return 'Nothing from the dictionary yet. Fetch again to look it up.';
    const t = title.trim();
    return t ? `Press Enter to look up “${t}”.` : 'Type it above, then press Enter to look it up.';
  });
  const meta = $derived.by(() => {
    if (!looksUp) return '';
    if (!dictionary) return dictionarySource;
    return `${dictionarySource} · fetched ${shortDate(iso(new Date(dictionary.fetchedAt)))}`;
  });

  // ---- saving a saved word's changes

  async function patch(change: WordPatch) {
    if (!start) return;
    const saved = await wordsAPI.update({ id: start.id, patch: change });
    title = saved.title;
    kind = saved.kind;
    await words.load();
  }

  // ---- the dictionary

  /** An idiom the dictionary knows as one becomes an idiom, unless the kind was picked. */
  function idiomFrom(found: Dictionary | null) {
    const saysIdiom = found?.partOfSpeech === idiomPartOfSpeech;
    if (saysIdiom && !kindSet) kind = WORD_KIND.idiom;
  }

  async function lookup() {
    const t = title.trim();
    if (!t || !looksUp || dupe) return;
    fetching = true;
    try {
      const found = await wordsAPI.lookup(t);
      dictionary = found;
      notFound = found === null;
      idiomFrom(found);
      if (!isNew) await patch({ dictionary: found });
    } finally {
      fetching = false;
    }
  }

  // A title handed over from Home is looked up once the kinds are in.
  let handed = isNew && handedTitle.trim() !== '';
  $effect(() => {
    if (!handed || !words.loaded) return;
    handed = false;
    untrack(lookup);
  });

  // ---- the title and kind

  function ontitle() {
    if (!isNew) return;
    ontyped?.(title);
    if (!kindSet) kind = guessKind(title);
    dictionary = null;
    notFound = false;
  }

  /** A saved word's title, on leaving the field: empty or taken goes back to what it was. */
  async function committitle() {
    if (!start) return;
    const next = savedTitle({ title, kind });
    const keep = !next || dupe !== undefined;
    if (keep) {
      title = start.title;
      return;
    }
    if (next !== start.title) await patch({ title: next });
  }

  function ontitlekey(e: KeyboardEvent) {
    if (e.key !== 'Enter' || e.metaKey || e.ctrlKey) return;
    e.preventDefault();
    if (isNew) lookup();
    else (e.currentTarget as HTMLInputElement).blur();
  }

  async function pickKind(next: string) {
    kindSet = true;
    kind = next;
    if (isNew) {
      dictionary = null;
      notFound = false;
      return;
    }
    await patch({ kind: next });
  }

  // ---- the notes and tags (a new word keeps them until it is added)

  async function savemeaning(text: string) {
    meaning = text;
    await patch({ meaning: text });
  }

  async function savenotes(text: string) {
    notes = text;
    await patch({ notes: text });
  }

  /** A saved word's chips follow the answer: a refused tag (a blocked one) never shows. */
  async function settags(next: string[]) {
    if (!start) {
      tags = next;
      return;
    }
    tags = await wordsAPI.setTags({ id: start.id, tags: next });
    // The list holds each word's tags too: this entry starts from them when it is opened again.
    await words.load();
  }

  // ---- adding and deleting

  const canAdd = $derived(title.trim() !== '' && !dupe);

  let adding = false;

  /** Saves the new word's tags. A refusal is returned, not thrown: the word exists by then and
   *  is opened first. */
  async function addTags(id: string): Promise<unknown> {
    if (!tags.length) return null;
    try {
      await wordsAPI.setTags({ id, tags });
      return null;
    } catch (e) {
      return e;
    }
  }

  async function add() {
    // A second ⌘↵ while the first add runs would hit the duplicate rule.
    if (!canAdd || adding) return;
    adding = true;
    let saved: Word;
    let tagsError: unknown;
    try {
      saved = await wordsAPI.create({ title, kind, dictionary, meaning, notes });
      tagsError = await addTags(saved.id);
    } finally {
      adding = false;
      await words.load();
    }
    // Also when only its tags were refused: the word is open, so a retry can't add it twice.
    onsaved(saved.id);
    if (tagsError) throw tagsError;
  }

  async function remove() {
    if (!start) return;
    await wordsAPI.remove(start.id);
    await words.load();
    toast.show('Word deleted');
    ondeleted();
  }

  function askRemove() {
    if (settings.asksDelete('word')) confirming = true;
    else remove();
  }

  function onpagekey(e: KeyboardEvent) {
    if (!isNew) return;
    const saves = e.key === 'Enter' && (e.metaKey || e.ctrlKey);
    if (saves) {
      e.preventDefault();
      add();
    } else if (e.key === 'Escape' && !e.defaultPrevented) {
      oncancel?.();
    }
  }

  function play() {
    if (!dictionary?.audio) return;
    // The recording is on the dictionary's server: offline, say so here, not as an app error.
    new Audio(dictionary.audio).play().catch(() => toast.show('The recording couldn’t be played'));
  }

  const kindLabel = $derived(kindInfo?.label ?? kind.toLowerCase());
  const showPos = $derived(Boolean(dictionary?.partOfSpeech) && dictionary?.partOfSpeech !== kindLabel);
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="page" onkeydown={onpagekey}>
  <PageBar>
    <Breadcrumbs crumbs={[{ label: kindLabel }, { label: isNew ? 'new' : start?.title || 'untitled' }]} />
    {#snippet actions()}
      {#if !isNew}
        <button class="icon-btn sm" aria-label="Delete word" title="Delete" onclick={askRemove}><Icon name="trash" size={14} /></button>
      {/if}
    {/snippet}
  </PageBar>

  <div class="doc">
    <div class="top">
      <input
        use:plainText
        use:focusWhen={isNew}
        class="title"
        aria-label="Title"
        placeholder="Something you came across…"
        bind:value={title}
        oninput={ontitle}
        onkeydown={ontitlekey}
        onblur={committitle} />
      <div class="meta-row">
        <KindPicker value={kind} kinds={words.kinds} onpick={pickKind} />
        {#if dictionary?.ipa}
          <span class="ipa">{dictionary.ipa}</span>
          {#if dictionary.audio}
            <button class="icon-btn sm" aria-label="Play pronunciation" title="Play pronunciation" onclick={play}><Icon name="sound" size={14} /></button>
          {/if}
        {/if}
        {#if showPos && dictionary?.partOfSpeech}<Chip text={dictionary.partOfSpeech} />{/if}
        <TagEditor {tags} onchange={settags} />
      </div>
      {#if dupe}
        <p class="dupe">Already saved. <button class="link-btn inline" onclick={() => onopen(dupe.id)}>Open “{dupe.title}”</button></p>
      {/if}
    </div>

    <WordDictionary
      {dictionary}
      {fetching}
      {hint}
      {meta}
      onrefetch={!isNew && looksUp ? lookup : undefined} />

    <Section title="What it is for me" indent={false}>
      <div class="notes">
        <WordNote label="Meaning" value={meaning} placeholder="What does it mean to you?" onsave={savemeaning} />
        <WordNote label="Notes" value={notes} placeholder="Examples, where you met it, anything else" onsave={savenotes} />
      </div>
    </Section>

    {#if isNew}
      <div class="adds">
        <button class="btn-primary" disabled={!canAdd} onclick={add}>Save</button>
        <button class="btn-ghost" onclick={oncancel}>Cancel</button>
      </div>
    {/if}
  </div>
</div>

{#if confirming}
  <ConfirmDialog
    title="Delete “{start?.title}”?"
    body="It goes from Words, with its meaning and notes."
    confirm="Delete"
    danger
    onconfirm={remove}
    onclose={() => (confirming = false)} />
{/if}

<style>
  .page {
    flex: 1 1 auto;
    min-width: 0;
    overflow: auto;
    padding: 0 24px;
  }

  .doc {
    max-width: 760px;
    margin: 0 auto;
    padding: 8px 0 48px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .top {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .title {
    width: 100%;
    padding: 0;
    border: 0;
    outline: none;
    background: transparent;
    font: var(--text-heading);
    color: var(--text);
    caret-color: var(--primary);
  }

  .title::placeholder {
    color: var(--c-muted-2);
  }

  .meta-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
  }

  .ipa {
    font: var(--text-mono);
    color: var(--text-muted);
  }

  .dupe {
    margin: 0;
    font: var(--text-caption);
    color: var(--text-muted);
  }

  .notes {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .adds {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-top: 4px;
  }
</style>
