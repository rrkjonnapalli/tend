<script lang="ts">
  // Home's add: one box for anything. It guesses where the text belongs as it is typed (the
  // Save-to switch overrides it); Continue (Enter) opens that area's own add with the text in.
  // Shift+Enter adds a line, Esc closes.
  import { goto } from '$app/navigation';
  import Modal from './Modal.svelte';
  import Segmented from './Segmented.svelte';
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { focusOnMount } from '#lib/actions/focus.action.ts';
  import { libraryAreas, saveTargets } from '#lib/config/library.config.ts';
  import type { SaveArea } from '#lib/enums/library.enum.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import { guessArea } from '#lib/util/library.ts';
  import { routes } from '#lib/util/routes.ts';

  let { onclose }: { onclose: () => void } = $props();

  const NO_GUESS = 'It picks for you as you type.';
  const options = saveTargets.map((t) => ({ value: t.area, label: t.label }));

  let draft = $state('');
  /** The area picked by hand; none follows the guess. */
  let picked = $state<SaveArea | null>(null);

  const guess = $derived(guessArea(draft));
  const target = $derived(picked ?? guess?.area ?? null);
  const why = $derived.by(() => {
    if (picked) return '';
    return guess?.why ?? NO_GUESS;
  });
  const hint = $derived.by(() => {
    if (!target) return '↵ continues where it belongs';
    const label = libraryAreas.find((a) => a.area === target)?.label;
    return `↵ continues in ${label}, with this filled in`;
  });

  function carryOn() {
    const text = draft.trim();
    if (!text || !target) return;
    library.handOff(text);
    goto(routes.libraryArea(target));
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== 'Enter' || e.shiftKey) return;
    e.preventDefault();
    carryOn();
  }
</script>

<Modal label="Add to the library" width={560} top={110} shade {onclose}>
  <div class="card">
    <textarea
      use:plainText
      use:focusOnMount
      class="cap"
      aria-label="What did you come across"
      placeholder="Paste a link, type a word, or write a snap…"
      bind:value={draft}
      {onkeydown}></textarea>
    <div class="dest">
      <span class="lbl">Save to</span>
      <!-- Nothing typed: no area is on yet. -->
      <Segmented {options} value={target} label="Save to" small onchange={(area) => (picked = area)} />
      <span class="why">{why}</span>
    </div>
    <div class="foot">
      <span class="hint">{hint}</span>
      <button class="btn-ghost" onclick={onclose}>Cancel</button>
      <button class="btn-primary" disabled={!guess} onclick={carryOn}>Continue</button>
    </div>
  </div>
</Modal>

<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 22px 24px 18px;
  }

  .cap {
    width: 100%;
    min-height: 56px;
    padding: 0;
    border: 0;
    outline: none;
    resize: none;
    field-sizing: content;
    max-height: 240px;
    background: transparent;
    font: 400 17px/26px var(--font-ui);
    color: var(--text);
    caret-color: var(--primary);
  }

  .cap::placeholder {
    color: var(--c-muted-2);
  }

  .dest {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
  }

  .lbl,
  .why {
    font: var(--text-caption);
    color: var(--text-muted);
  }

  .foot {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .hint {
    margin-right: auto;
    font: var(--text-meta);
    color: var(--text-muted);
  }
</style>
