<script lang="ts">
  import Chip from './Chip.svelte';
  import { plainText } from '#lib/actions/plain-text.action.ts';
  // Tag chips with a red ✕ each, and an inline input that suggests tags from the shared pool.
  import Popover from './Popover.svelte';
  import { focusOnMount } from '#lib/actions/focus.action.ts';
  import { slug } from '#lib/util/names.ts';
  import { tagPool } from '#lib/stores/tags.svelte.ts';
  import { nextIndex, prevIndex } from '#lib/util/list-nav.ts';

  let {
    tags,
    onchange,
    addLabel = '+ tag',
  }: { tags: string[]; onchange: (tags: string[]) => void; addLabel?: string } = $props();

  const MAX = 6;
  const PICK_KEYS = ['Enter', ',', 'Tab'];
  let adding = $state(false);
  let draft = $state('');
  let hi = $state(0);
  let input = $state<HTMLInputElement>();

  // A tag is its slug ("UI Design" → ui_design), the same on every device.
  const query = $derived(slug(draft));
  const have = $derived(new Set(tags));

  const options = $derived.by(() => {
    const isSuggestion = (t: string) => !have.has(t) && (!query || t.includes(query));
    const matches = tagPool.names.filter(isSuggestion).slice(0, MAX);
    const out = matches.map((t) => ({ label: t, value: t, isNew: false }));
    const exists = tagPool.names.includes(query) || have.has(query);
    if (query && !exists) out.push({ label: `Create “${query}”`, value: query, isNew: true });
    return out;
  });

  function add(value: string) {
    const v = slug(value);
    clearDraft();
    if (!v || have.has(v)) return;
    tagPool.learn([v]);
    onchange([...tags, v]);
  }

  function remove(tag: string) {
    onchange(tags.filter((t) => t !== tag));
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
    const highlighted = options[Math.min(hi, options.length - 1)];
    add(highlighted.value);
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
      onchange(tags.slice(0, -1));
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
    {#if options.length}
      <Popover anchor={input} role="listbox" label="Tag suggestions" pad onclose={stop}>
        <div class="sugg">
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
</style>
