<script lang="ts">
  // Tag chips with a red ✕ each, and an inline input that suggests tags from the shared pool.
  // The same editor for notes, tasks, words, snaps and references. A blocked tag is never
  // suggested and can't be added: typed by hand, the list says so and the chips stay as they were.
  import Chip from './Chip.svelte';
  import Popover from './Popover.svelte';
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { focusOnMount } from '#lib/actions/focus.action.ts';
  import { toAppError } from '#lib/errors/app-error.ts';
  import { errors } from '#lib/errors/registry.ts';
  import { slug } from '#lib/util/names.ts';
  import { tagPool } from '#lib/stores/tags.svelte.ts';
  import { nextIndex, prevIndex } from '#lib/util/list-nav.ts';

  let {
    tags,
    onchange,
    addLabel = '+ tag',
  }: {
    tags: string[];
    /** Takes the new tags. A caller that saves at once returns its promise and sets `tags` from
     *  the answer, so a refused tag never shows as a chip. */
    onchange: (tags: string[]) => unknown;
    addLabel?: string;
  } = $props();

  const MAX = 6;
  const PICK_KEYS = ['Enter', ',', 'Tab'];
  let adding = $state(false);
  let draft = $state('');
  let hi = $state(0);
  let input = $state<HTMLInputElement>();

  // A tag is its slug ("UI Design" → ui_design), the same on every device.
  const query = $derived(slug(draft));
  const have = $derived(new Set(tags));
  const blocked = $derived(tagPool.isBlocked(query));

  const options = $derived.by(() => {
    const isSuggestion = (t: string) => !have.has(t) && (!query || t.includes(query));
    const matches = tagPool.names.filter(isSuggestion).slice(0, MAX);
    const out = matches.map((t) => ({ label: t, value: t, isNew: false }));
    const exists = tagPool.names.includes(query) || have.has(query);
    const creatable = query !== '' && !exists && !blocked;
    if (creatable) out.push({ label: `Create “${query}”`, value: query, isNew: true });
    return out;
  });

  /** Hands the caller the new tags. A refusal for a tag blocked since the pool was loaded
   *  (TAG_BLOCKED) means the pool is stale: it is loaded again, and the error goes on to the
   *  global handler. */
  async function change(next: string[]) {
    try {
      await onchange(next);
    } catch (e) {
      if (toAppError(e).is(errors.TAG_BLOCKED)) await tagPool.load();
      throw e;
    }
  }

  async function add(value: string) {
    const v = slug(value);
    // A blocked tag keeps its text in the field, under which the list says why.
    if (tagPool.isBlocked(v)) return;
    clearDraft();
    if (!v || have.has(v)) return;
    await change([...tags, v]);
    tagPool.learn([v]);
  }

  function remove(tag: string) {
    return change(tags.filter((t) => t !== tag));
  }

  export function start() {
    adding = true;
    clearDraft();
  }

  function stop() {
    adding = false;
    clearDraft();
  }

  function clearDraft() {
    draft = '';
    hi = 0;
  }

  /** Takes the highlighted suggestion, or the typed text when there is none. */
  function pick() {
    if (!options.length) return add(draft);
    const highlighted = options[hi] ?? options[options.length - 1];
    return add(highlighted.value);
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      stop();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      hi = nextIndex(hi, options.length);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      hi = prevIndex(hi);
    } else if (PICK_KEYS.includes(e.key)) {
      if (!options.length && !query) return;
      e.preventDefault();
      pick();
    } else if (e.key === 'Backspace' && !draft && tags.length) {
      change(tags.slice(0, -1));
    }
  }

  // A comma ends a tag, so it never stays in the text.
  function oninput() {
    draft = draft.replace(/,/g, '');
    hi = 0;
  }

  function choose(e: MouseEvent, value: string) {
    // Keep focus in the input so its blur doesn't close the list first.
    e.preventDefault();
    add(value);
  }
</script>

<div class="tags" aria-label="Tags">
  {#each tags as tag (tag)}
    <Chip text={tag} removeLabel="Remove tag {tag}" onremove={() => remove(tag)} />
  {/each}
  {#if adding}
    <input
      use:plainText
      bind:this={input}
      class="in"
      aria-label="New tag"
      placeholder="tag"
      bind:value={draft}
      {oninput}
      {onkeydown}
      onblur={stop}
      use:focusOnMount />
    {#if options.length || blocked}
      <Popover anchor={input} role="listbox" label="Tag suggestions" pad onclose={stop}>
        <div class="sugg">
          {#if blocked}<span class="refused" role="alert">{errors.TAG_BLOCKED.message}</span>{/if}
          {#each options as o, i (o.value + o.isNew)}
            <button
              class="opt"
              class:on={i === hi}
              class:new={o.isNew}
              role="option"
              aria-selected={i === hi}
              onmousedown={(e) => choose(e, o.value)}>{o.label}</button>
          {/each}
        </div>
      </Popover>
    {/if}
  {:else}
    <button class="add" aria-label="Add tag" title="Add tag" onclick={start}>{addLabel}</button>
  {/if}
</div>

<style>
  .tags {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    min-height: 26px;
  }

  .in {
    width: 96px;
    height: 22px;
    padding: 0 8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: var(--c-hover);
    outline: none;
    font: var(--text-caption);
  }

  .in::placeholder {
    color: var(--c-muted-2);
  }

  .add {
    height: 22px;
    padding: 0 6px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--c-muted-2);
    font: var(--text-caption);
    cursor: pointer;
  }

  .add:hover {
    background: var(--c-hover);
    color: var(--text);
  }

  .sugg {
    min-width: 160px;
    display: flex;
    flex-direction: column;
  }

  .refused {
    padding: 4px 12px;
    font: var(--text-meta);
    color: var(--danger);
  }
</style>
