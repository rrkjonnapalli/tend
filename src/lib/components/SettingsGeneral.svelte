<script lang="ts">
  // Settings → General: the theme, the file size limit, the timer sound, delete confirmations, updates, the vault folder, and notes or folders skipped for breaking
  // the name rule (with the name each can be renamed to, and one button to rename them all).
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
  import { deleteChoices, fileLimitRange, settingDefaults, settingKeys, themeChoices } from '#lib/config/settings.config.ts';
  import NumberSetting from './NumberSetting.svelte';
  import { notes } from '#lib/stores/notes.svelte.ts';
  import type { ThemePref } from '#lib/types/theme.type.ts';
  import { routes } from '#lib/util/routes.ts';
  import { update } from '#lib/stores/update.svelte.ts';

  let skipped = $state<Skipped[]>([]);
  let showSkipped = $state(false);
  const toFix = $derived(fixable(skipped).length);

  const theme = $derived(settings.get<ThemePref>(settingKeys.theme, settingDefaults.theme));
  const fileLimit = $derived(settings.get<number>(settingKeys.fileLimitMB, settingDefaults.fileLimitMB));

  onMount(loadSkipped);

  /** Saves the limit, then rescans so notes over (or now under) it leave (or join) the index. */
  async function setFileLimit(mb: number) {
    await settings.set(settingKeys.fileLimitMB, mb);
    await notes.rescan();
  }

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
  <div class="line">
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
  <h2 class="section-title">Files</h2>
  <NumberSetting
    value={fileLimit}
    min={fileLimitRange.min}
    max={fileLimitRange.max}
    shown="{fileLimit} MB"
    unit="MB · {fileLimitRange.min} to {fileLimitRange.max}"
    label="File size limit in MB"
    what="file size limit"
    onsave={setFileLimit} />
  <span class="hint">
    Notes this size or larger aren’t indexed, files this size can’t be attached, and sync never commits one.
    Lowering it never deletes anything: a note Tend already has stays, but its changes aren’t picked up while it’s too big.
  </span>
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

<section class="group">
  <h2 class="section-title">Confirmations</h2>
  {#each deleteChoices as c (c.kind)}
    <label class="check">
      <input
        type="checkbox"
        checked={settings.asksDelete(c.kind)}
        onchange={(e) => settings.set(settingKeys.confirmDelete(c.kind), e.currentTarget.checked)} />
      {c.label}
    </label>
  {/each}
  <span class="hint">Deleting a folder, a status or a label always asks.</span>
</section>

<section class="group">
  <h2 class="section-title">Updates</h2>
  <div class="line">
    <span>Tend {update.current}</span>
    {#if update.available}
      <button class="btn-ghost" disabled={update.installing} onclick={update.install}>
        {update.installing ? 'Installing…' : `Install ${update.available.version} and restart`}
      </button>
    {:else}
      <button class="btn-ghost" disabled={update.checking} onclick={update.check}>
        {update.checking ? 'Checking…' : 'Check for updates'}
      </button>
    {/if}
  </div>
  {#if update.available}
    <span class="hint">Tend saves your work and syncs first, then opens again as the new version.</span>
  {:else if update.checked}
    <span class="hint">Tend is up to date.</span>
  {/if}
</section>

<style>
  .group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .line {
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
