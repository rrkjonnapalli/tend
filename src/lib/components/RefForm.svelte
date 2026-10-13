<script lang="ts">
  // The one popup for adding and editing a reference. Only the reference is needed: paste,
  // Enter, done. A link gets its page title fetched into Title unless one was typed; text is its
  // own title until changed. Link or text is picked from what is typed, until picked by hand.
  import { untrack } from 'svelte';
  import Modal from './Modal.svelte';
  import Segmented from './Segmented.svelte';
  import TagEditor from './TagEditor.svelte';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import Icon from './Icon.svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { focusOnMount } from '#lib/actions/focus.action.ts';
  import { refsAPI } from '#lib/api/refs.ts';
  import { titleFetchDelayMs } from '#lib/config/library.config.ts';
  import { REF_KIND, type RefKind } from '#lib/enums/ref.enum.ts';
  import { refs } from '#lib/stores/refs.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import type { NewRef, Ref } from '#lib/types/ref.type.ts';
  import { iso, shortDate } from '#lib/util/dates.ts';
  import { kindOf, openableUrl, siteOf, urlKey } from '#lib/util/refs.ts';

  let {
    editing,
    filled = '',
    onclose,
    onopen,
  }: {
    /** The reference to edit; none adds a new one. */
    editing: Ref | null;
    /** A new reference's text, handed over from Home. */
    filled?: string;
    onclose: () => void;
    /** Opens another reference (the one a duplicate link points at). */
    onopen: (ref: Ref) => void;
  } = $props();

  const KINDS = [
    { value: REF_KIND.link, label: 'link' },
    { value: REF_KIND.text, label: 'text' },
  ];

  // The popup is remounted for each reference, so these start from the one it opened with.
  const start = untrack(() => editing);
  const handed = untrack(() => filled);
  let value = $state(start?.value ?? handed);
  let kind = $state<RefKind>(start?.kind ?? REF_KIND.text);
  let kindSet = $state(start !== null);
  let title = $state(start?.title ?? start?.value ?? '');
  let titleSet = $state(Boolean(start?.title));
  let why = $state(start?.why ?? '');
  let tags = $state<string[]>(start?.tags ?? []);
  let fetching = $state(false);
  let confirming = $state(false);

  const isLink = $derived(kind === REF_KIND.link);
  const dupe = $derived(isLink && value.trim() ? refs.withKey(urlKey(value), start?.id) : undefined);
  const canSave = $derived(value.trim() !== '' && !dupe);

  // ---- the page title

  let timer: ReturnType<typeof setTimeout> | undefined;
  /** Each fetch's number; only the latest one may fill Title. */
  let asked = 0;
  /** The fetch under way, so a link saved before it ends still gets its title. */
  let pending: Promise<string | null> | null = null;

  /** No title is a fine outcome (no page, offline): the reference just shows its link. */
  const fetchTitle = (url: string) => refsAPI.title(url).catch(() => null);

  function stopFetch() {
    clearTimeout(timer);
    asked += 1;
    fetching = false;
    pending = null;
  }

  /** A link whose title wasn't typed gets it fetched once typing pauses. */
  function scheduleFetch() {
    stopFetch();
    const wanted = isLink && !titleSet && value.trim() !== '';
    if (!wanted) return;
    fetching = true;
    const url = value;
    const mine = asked;
    timer = setTimeout(() => {
      pending = fetchTitle(url);
      pending.then((fetched) => {
        if (mine !== asked) return;
        fetching = false;
        if (!titleSet) title = fetched ?? '';
      });
    }, titleFetchDelayMs);
  }

  $effect(() => () => clearTimeout(timer));

  // ---- typing

  function onvalue() {
    if (!kindSet) kind = kindOf(value);
    if (!titleSet) title = isLink ? '' : value;
    scheduleFetch();
  }

  /** Picked by hand: it stays. Text takes its title from the reference; a link fetches one. */
  function setKind(next: RefKind) {
    kindSet = true;
    if (next === kind) return;
    kind = next;
    if (!titleSet) title = isLink ? '' : value;
    scheduleFetch();
  }

  function ontitle() {
    titleSet = title.trim() !== '';
    stopFetch();
  }

  // ---- saving

  /** The title stored: typed, or fetched for a link; text without its own title stores none. */
  function savedTitle(): string | null {
    const typed = title.trim();
    if (titleSet || isLink) return typed || null;
    return null;
  }

  /** A link saved before its title arrived gets it when the fetch ends. */
  function titleLater(saved: Ref) {
    if (!isLink || saved.title) return;
    const late = pending ?? fetchTitle(saved.value);
    late.then(async (fetched) => {
      if (!fetched) return;
      await refsAPI.update({ id: saved.id, patch: { title: fetched } });
      await refs.load();
    });
  }

  /** Writes the reference, then its tags. */
  async function write(): Promise<Ref> {
    const input: NewRef = { kind, value: value.trim(), title: savedTitle(), why };
    const saved = start
      ? await refsAPI.update({ id: start.id, patch: input })
      : await refsAPI.create(input);
    const tagsChanged = tags.join(' ') !== saved.tags.join(' ');
    if (tagsChanged) await refsAPI.setTags({ id: saved.id, tags });
    return saved;
  }

  let saving = false;

  async function save() {
    // A second Enter while the first save runs would write the reference twice.
    if (!canSave || saving) return;
    saving = true;
    let saved: Ref;
    try {
      saved = await write();
    } finally {
      saving = false;
      // Also after a failure: the reference may be saved with only its tags missing.
      await refs.load();
    }
    titleLater(saved);
    clearTimeout(timer);
    toast.show(start ? 'Saved' : 'Saved to references');
    onclose();
  }

  async function remove() {
    if (!start) return;
    await refsAPI.remove(start.id);
    await refs.load();
    toast.show('Reference deleted');
    onclose();
  }

  function askRemove() {
    if (settings.asksDelete('reference')) confirming = true;
    else remove();
  }

  function onvaluekey(e: KeyboardEvent) {
    if (e.key !== 'Enter' || e.shiftKey) return;
    e.preventDefault();
    save();
  }

  function onformkey(e: KeyboardEvent) {
    const saves = e.key === 'Enter' && (e.metaKey || e.ctrlKey);
    if (!saves) return;
    e.preventDefault();
    save();
  }

  // Text handed over from Home counts as typed: it picks link or text and starts the title fetch.
  if (handed) onvalue();

  const titleHint = $derived.by(() => {
    if (!isLink) return 'Same as the reference';
    return fetching ? 'Getting the page title…' : 'Page title';
  });
  const savedOn = $derived(dupe ? shortDate(iso(new Date(dupe.createdAt))) : '');
</script>

<Modal label={start ? 'Edit reference' : 'Add a reference'} width={560} top={96} shade {onclose}>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="card" onkeydown={onformkey}>
    <input
      use:plainText
      use:focusOnMount
      class="main"
      aria-label="Reference: a link or text"
      placeholder="Paste a link, or type a book, film, video…"
      bind:value
      oninput={onvalue}
      onkeydown={onvaluekey} />
    <div class="under"><Segmented options={KINDS} value={kind} label="Type" onchange={setKind} /></div>
    {#if dupe}
      <p class="dupe">You saved this link on {savedOn}. <button class="link" onclick={() => onopen(dupe)}>Open it</button></p>
    {/if}

    <label class="field">
      <span class="lbl">Title</span>
      <span class="title-row">
        <input use:plainText class="title" placeholder={titleHint} bind:value={title} oninput={ontitle} />
        {#if fetching}<span class="dots" aria-hidden="true"><i></i><i></i><i></i></span>{/if}
        {#if isLink && value.trim()}
          <span class="site">{siteOf(value)}</span>
          <button class="icon-btn sm" aria-label="Open link" title="Open link" onclick={() => openUrl(openableUrl(value))}>
            <Icon name="open" size={13} />
          </button>
        {/if}
      </span>
    </label>

    <label class="field">
      <span class="lbl">Why I kept it</span>
      <textarea use:plainText class="why" rows="2" placeholder="Optional. A few words now saves a mystery later." bind:value={why}></textarea>
    </label>

    <div class="field">
      <span class="lbl">Tags</span>
      <TagEditor {tags} onchange={(next) => (tags = next)} />
    </div>

    <div class="foot">
      {#if start}
        <button class="del" onclick={askRemove}><Icon name="trash" size={14} />Delete</button>
      {/if}
      <span class="grow"></span>
      <span class="hint">{start ? '⌘↵ saves' : '↵ saves'}</span>
      <button class="btn-ghost" onclick={onclose}>Cancel</button>
      <button class="btn-primary" disabled={!canSave} onclick={save}>Save</button>
    </div>
  </div>
</Modal>

{#if confirming}
  <ConfirmDialog
    title="Delete this reference?"
    body="It goes from References and from every tag."
    confirm="Delete"
    danger
    onconfirm={remove}
    onclose={() => (confirming = false)} />
{/if}

<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 22px 24px 18px;
  }

  .main {
    width: 100%;
    padding: 0;
    border: 0;
    outline: none;
    background: transparent;
    font: 500 20px/28px var(--font-heading);
    color: var(--text);
    caret-color: var(--primary);
  }

  .main::placeholder,
  .title::placeholder,
  .why::placeholder {
    color: var(--c-muted-2);
  }

  .under {
    display: flex;
  }

  .dupe {
    margin: 0;
    font: var(--text-caption);
    color: var(--text-muted);
  }

  .link {
    padding: 0;
    border: 0;
    background: transparent;
    font: inherit;
    color: var(--primary);
    cursor: pointer;
  }

  .link:hover,
  .link:focus-visible {
    color: var(--primary-hover);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .lbl {
    font: 700 12px/16px var(--font-ui);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .title,
  .why {
    flex: 1 1 auto;
    min-width: 0;
    padding: 0;
    border: 0;
    outline: none;
    background: transparent;
    font: 400 15px/24px var(--font-ui);
    color: var(--text);
  }

  .why {
    resize: none;
  }

  .site {
    flex: none;
    font: var(--text-meta);
    font-family: var(--font-mono);
    color: var(--c-muted-2);
  }

  .dots {
    display: inline-flex;
    gap: 3px;
  }

  .dots i {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--c-faint);
    animation: blink 1s infinite;
  }

  .dots i:nth-child(2) {
    animation-delay: 0.15s;
  }

  .dots i:nth-child(3) {
    animation-delay: 0.3s;
  }

  @keyframes blink {
    50% {
      opacity: 0.25;
    }
  }

  .foot {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-top: 4px;
  }

  .grow {
    flex: 1;
  }

  .hint {
    font: var(--text-meta);
    color: var(--c-muted-2);
  }

  .del {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 28px;
    padding: 0 8px 0 4px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    font: var(--text-caption);
    color: var(--c-muted-2);
    cursor: pointer;
  }

  .del:hover,
  .del:focus-visible {
    background: var(--danger-soft);
    color: var(--danger);
  }
</style>
