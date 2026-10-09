<script lang="ts">
  // Settings → General: the theme, the timer sound, the vault folder, and notes or folders skipped for breaking the name rule
  // (with the name each can be renamed to, and one button to rename them all).
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import Icon from './Icon.svelte';
  import { notesAPI } from '#lib/api/notes.ts';
  import { vault } from '#lib/stores/vault.svelte.ts';
  import type { Skipped } from '#lib/types/note.type.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { plural } from '#lib/util/plural.ts';
  import { fixable } from '#lib/util/skipped.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { settingDefaults, settingKeys, themeChoices } from '#lib/config/settings.config.ts';
  import type { ThemePref } from '#lib/types/theme.type.ts';
  import { routes } from '#lib/util/routes.ts';

  let skipped = $state<Skipped[]>([]);
  let showSkipped = $state(false);
  const toFix = $derived(fixable(skipped).length);

  const theme = $derived(settings.get<ThemePref>(settingKeys.theme, settingDefaults.theme));

  onMount(loadSkipped);

  async function loadSkipped() {
    skipped = await notesAPI.skipped();
  }

  function toggleSkipped() {
    showSkipped = !showSkipped;
  }

  async function fixNames() {
    const left = await notesAPI.fixNames();
    toast.show(`Renamed ${plural(skipped.length - left.length, 'item')}`);
    skipped = left;
  }
</script>

<section class="group">
  <h2 class="section-title">Vault</h2>
  <div class="vault">
    <span class="vpath">{vault.path}</span>
    <button class="link-btn" onclick={() => goto(routes.start)}>Change vault folder…</button>
  </div>
  {#if skipped.length}
    <button class="skipped" aria-expanded={showSkipped} onclick={toggleSkipped}>
      <Icon name={showSkipped ? 'chevronDown' : 'chevronRight'} size={10} stroke={3} />
      {plural(skipped.length, 'item')} skipped: names must be lowercase letters, digits and _
    </button>
    {#if showSkipped}
      <ul class="skiplist">
        {#each skipped as item (item.path)}
          <li>{item.path} <span class="to">{item.fix ? `→ ${item.fix}` : '· no free name'}</span></li>
        {/each}
      </ul>
      {#if toFix}
        <div><button class="link-btn" onclick={fixNames}>Rename {plural(toFix, 'item')} to fit</button></div>
      {/if}
    {/if}
  {/if}
</section>

<section class="group">
  <h2 class="section-title">Appearance</h2>
  <div class="seg" role="radiogroup" aria-label="Theme">
    {#each themeChoices as t (t.value)}
      <label class="choice" class:on={theme === t.value}>
        <input type="radio" name="theme" value={t.value} checked={theme === t.value} onchange={() => settings.set(settingKeys.theme, t.value)} />
        {t.label}
      </label>
    {/each}
  </div>
  <span class="hint">System follows your Mac's light or dark setting.</span>
</section>

<section class="group">
  <h2 class="section-title">Timers</h2>
  <label class="check">
    <input
      type="checkbox"
      checked={settings.get(settingKeys.timersSound, settingDefaults.timersSound)}
      onchange={(e) => settings.set(settingKeys.timersSound, e.currentTarget.checked)} />
    Play a sound when a timer ends
  </label>
</section>

<style>
  .group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .vault {
    display: flex;
    align-items: center;
    gap: 16px;
    min-height: 32px;
  }

  .vpath {
    flex: 1;
    min-width: 0;
    font: var(--text-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .skipped {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0;
    border: 0;
    background: transparent;
    font: var(--text-caption);
    color: var(--c-muted-2);
    cursor: pointer;
    text-align: left;
  }

  .skipped:hover {
    color: var(--text);
  }

  .skiplist {
    margin: 0;
    padding: 0 0 0 16px;
    list-style: none;
    font: var(--text-path);
    color: var(--text-muted);
  }

  .to {
    color: var(--c-muted-2);
  }

  /* Theme: a segmented control; the radios stay for the keyboard (←→) and screen readers. */
  .seg {
    display: inline-flex;
    align-self: flex-start;
    gap: 2px;
    padding: 2px;
    border-radius: var(--radius-control);
    background: var(--c-chip);
  }

  .choice {
    position: relative;
    display: inline-flex;
    align-items: center;
    height: 26px;
    padding: 0 12px;
    border-radius: var(--radius-sm);
    font: var(--text-caption);
    color: var(--c-muted-2);
    cursor: pointer;
  }

  .choice:hover {
    color: var(--text);
  }

  .choice.on {
    background: var(--c-page);
    color: var(--text);
  }

  .choice:has(input:focus-visible) {
    background: var(--c-hover);
    color: var(--text);
  }

  .choice input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .hint {
    font: var(--text-meta);
    color: var(--c-muted-2);
  }

  .check {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font: var(--text-ui);
    cursor: pointer;
  }

  .check input {
    accent-color: var(--primary);
  }

</style>
