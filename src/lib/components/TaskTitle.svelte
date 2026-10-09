<script lang="ts">
  import { plainText } from '#lib/actions/plain-text.action.ts';
  // The task title: a borderless box that grows over several lines and saves on blur or Enter.
  let { value, onsave }: { value: string; onsave: (title: string) => void } = $props();

  let title = $state('');
  $effect(() => {
    title = value;
  });

  /** Grows the title box to fit long titles over several lines. */
  function autogrow(node: HTMLTextAreaElement, _value: string) {
    const fit = () => {
      node.style.height = 'auto';
      node.style.height = `${node.scrollHeight}px`;
    };
    const ro = new ResizeObserver(fit);
    ro.observe(node);
    fit();
    return { update: fit, destroy: () => ro.disconnect() };
  }

  /** A title is one line: pasted line breaks become spaces. */
  function joinLines() {
    title = title.replace(/\s*\n\s*/g, ' ');
  }

  /** An emptied title goes back to the saved one. */
  function save() {
    const next = title.trim();
    if (next === value) return;
    if (!next) title = value;
    else onsave(next);
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== 'Enter') return;
    e.preventDefault();
    (e.currentTarget as HTMLTextAreaElement).blur();
  }
</script>

<textarea use:plainText class="title" data-zone="field" aria-label="Title" rows="1" bind:value={title} use:autogrow={title} oninput={joinLines} onblur={save} {onkeydown}></textarea>

<style>
  .title {
    display: block;
    width: 100%;
    padding: 0;
    border: 0;
    outline: none;
    resize: none;
    overflow: hidden;
    background: transparent;
    font: var(--text-heading);
    color: var(--text);
  }
</style>
