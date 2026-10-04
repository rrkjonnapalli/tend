<script lang="ts">
  // Tag chips with a red ✕ each, and an inline input that suggests tags from the shared pool.
  import Icon from './Icon.svelte';
  import Popover from './Popover.svelte';
  import { tagPool } from '#lib/stores/tags.svelte.ts';

  let {
    tags,
    onchange,
    addLabel = '+ tag',
  }: { tags: string[]; onchange: (tags: string[]) => void; addLabel?: string } = $props();

  const MAX = 6;
  let adding = $state(false);
  let draft = $state('');
  let hi = $state(0);
  let input = $state<HTMLInputElement>();

  const lower = (s: string) => s.toLowerCase();
  const query = $derived(lower(draft.trim()));
  const options = $derived.by(() => {
    const have = new Set(tags.map(lower));
    const matches = tagPool.names.filter((t) => !have.has(lower(t)) && (!query || lower(t).includes(query))).slice(0, MAX);
    const exists = tagPool.names.some((t) => lower(t) === query) || have.has(query);
    const out = matches.map((t) => ({ label: t, value: t, isNew: false }));
    if (query && !exists) out.push({ label: `Create “${draft.trim()}”`, value: draft.trim(), isNew: true });
    return out;
  });

  function add(value: string) {
    const v = value.trim();
    draft = '';
    hi = 0;
    if (!v || tags.some((t) => lower(t) === lower(v))) return;
    tagPool.learn([v]);
    onchange([...tags, v]);
  }

  function start() {
    adding = true;
    draft = '';
    hi = 0;
  }

  function stop() {
    adding = false;
    draft = '';
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      stop();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      hi = Math.min(hi + 1, options.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      hi = Math.max(hi - 1, 0);
    } else if (e.key === 'Enter' || e.key === ',' || e.key === 'Tab') {
      if (!options.length && !query) return;
      e.preventDefault();
      add(options.length ? options[Math.min(hi, options.length - 1)].value : draft);
    } else if (e.key === 'Backspace' && !draft && tags.length) {
      onchange(tags.slice(0, -1));
    }
  }

  function focusOnMount(node: HTMLInputElement) {
    node.focus();
  }
</script>

<div class="tags" aria-label="Tags">
  {#each tags as tag (tag)}
    <span class="chip" title={tag}>
      <span class="tl">{tag}</span>
      <button class="x" aria-label="Remove tag {tag}" title="Remove" onclick={() => onchange(tags.filter((t) => t !== tag))}>
        <Icon name="close" size={9} stroke={3} />
      </button>
    </span>
  {/each}
  {#if adding}
    <input
      bind:this={input}
      class="in"
      aria-label="New tag"
      placeholder="tag"
      bind:value={draft}
      oninput={() => ((draft = draft.replace(/,/g, '')), (hi = 0))}
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
              onmousedown={(e) => (e.preventDefault(), add(o.value))}>{o.label}</button>
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

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    max-width: 152px;
    padding: 2px 4px 2px 8px;
    border-radius: var(--radius-sm);
    background: var(--c-hover);
    color: var(--text-muted);
    font: 400 13px/18px var(--font-ui);
  }

  .tl {
    min-width: 0;
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .x {
    flex: 0 0 14px;
    width: 14px;
    height: 14px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--danger);
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .x:hover {
    background: var(--danger-soft);
  }

  .in {
    width: 96px;
    height: 22px;
    padding: 0 8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: var(--c-hover);
    outline: none;
    font: 400 13px/18px var(--font-ui);
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
    font: 400 13px/18px var(--font-ui);
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
