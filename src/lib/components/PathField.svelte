<script lang="ts">
  import { plainText } from '#lib/actions/plain-text.action.ts';
  // A folder path input with suggestions as you type. Tab or Enter takes the highlighted one.
  import { cleanPath } from '#lib/util/names.ts';
  import { nextIndex, prevIndex } from '#lib/util/list-nav.ts';

  let {
    value = $bindable(''),
    folders,
    onenter,
    label = 'Path',
  }: {
    value: string;
    folders: string[];
    /** Enter with no suggestion to take (move on to the next field). */
    onenter: () => void;
    label?: string;
  } = $props();

  const MAX_SUGGESTIONS = 6;

  let focused = $state(false);
  let hi = $state(0);
  const query = $derived(cleanPath(value));
  const suggestions = $derived(folders.filter((f) => f && f !== query && f.includes(query)).slice(0, MAX_SUGGESTIONS));
  const showing = $derived(focused && suggestions.length > 0);

  function take(path: string) {
    value = path;
    hi = 0;
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown' && showing) {
      e.preventDefault();
      hi = nextIndex(hi, suggestions.length);
    } else if (e.key === 'ArrowUp' && showing) {
      e.preventDefault();
      hi = prevIndex(hi);
    } else if ((e.key === 'Tab' || e.key === 'Enter') && showing) {
      e.preventDefault();
      take(suggestions[hi]);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      onenter();
    }
  }

  /** mousedown, not click, so the input keeps focus and the list stays open. */
  function onpick(e: MouseEvent, path: string) {
    e.preventDefault();
    take(path);
  }
</script>

<span class="wrap">
  <input
    use:plainText
    class="path"
    aria-label={label}
    placeholder="top level"
    spellcheck="false"
    autocomplete="off"
    bind:value
    oninput={() => (hi = 0)}
    onfocus={() => (focused = true)}
    onblur={() => (focused = false)}
    {onkeydown} />
  {#if showing}
    <div class="sugg" role="listbox" aria-label="Folders">
      {#each suggestions as s, i (s)}
        <button class="sg" class:on={i === hi} role="option" aria-selected={i === hi} onmousedown={(e) => onpick(e, s)}>{s}</button>
      {/each}
    </div>
  {/if}
</span>

<style>
  .wrap {
    position: relative;
    flex: 1;
    min-width: 0;
    display: flex;
  }

  .path {
    flex: 1;
    min-width: 0;
    height: 28px;
    border: 0;
    outline: none;
    background: transparent;
    font: var(--text-code);
    color: var(--text-muted);
  }

  .path::placeholder {
    color: var(--c-faint);
  }

  .sugg {
    position: absolute;
    left: -10px;
    right: 0;
    top: 34px;
    z-index: 3;
    padding: 4px 0;
    background: var(--c-page);
    border: 1px solid var(--border);
    border-radius: 6px;
  }

  .sg {
    display: block;
    width: 100%;
    height: 28px;
    padding: 0 10px;
    border: 0;
    background: transparent;
    text-align: left;
    font: var(--text-mono);
    color: var(--text-muted);
    cursor: pointer;
  }

  .sg.on,
  .sg:hover {
    background: var(--c-hover);
    color: var(--text);
  }
</style>
