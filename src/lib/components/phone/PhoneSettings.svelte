<script lang="ts">
  // The top of Settings › General on a phone (MD-Settings): the vault, sync, appearance (theme and
  // the menu button's side) and fonts. The sections are in the drawer. Spec: tend-mobile-spec.md §11.
  import { onMount } from 'svelte';
  import ChoicePicker from '../ChoicePicker.svelte';
  import { appAPI } from '#lib/api/app.ts';
  import { vault } from '#lib/stores/vault.svelte.ts';
  import { sync } from '#lib/stores/sync.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { settingDefaults, settingKeys, textSizeChoices, themeChoices } from '#lib/config/settings.config.ts';
  import { SETTINGS_SECTION } from '#lib/enums/settings.enum.ts';
  import type { ThemePref } from '#lib/types/theme.type.ts';
  import type { TextSize } from '#lib/types/font.type.ts';
  import type { MenuSide } from '#lib/types/ui.type.ts';
  import { ago } from '#lib/util/dates.ts';
  import { routes } from '#lib/util/routes.ts';
  import { repoLabel } from '#lib/util/sync.ts';

  const sides: { value: MenuSide; label: string }[] = [
    { value: 'left', label: 'Left' },
    { value: 'right', label: 'Right' },
  ];

  /** The families installed on this phone, for the font lists. */
  let fonts = $state<string[]>([]);
  onMount(async () => {
    fonts = await appAPI.fonts();
  });

  const theme = $derived(settings.get<ThemePref>(settingKeys.theme, settingDefaults.theme));
  const side = $derived(settings.get<MenuSide>(settingKeys.menuSide, settingDefaults.menuSide));
  const appFont = $derived(settings.get<string>(settingKeys.appFont, settingDefaults.appFont));
  const editorFont = $derived(settings.get<string>(settingKeys.editorFont, settingDefaults.editorFont));
  const textSize = $derived(settings.get<TextSize>(settingKeys.textSize, settingDefaults.textSize));
  const config = $derived(sync.status?.config ?? null);
  const repo = $derived(config ? repoLabel(config.url) : '');
  const lastOk = $derived(sync.status?.lastOkAt ?? null);
  const syncLine = $derived(lineOf(lastOk));

  function lineOf(last: number | null) {
    if (sync.state === 'running') return 'Syncing…';
    if (last) return `Synced ${ago(last)}`;
    return 'Not synced yet';
  }

  /** The font list: Tend's own first, then the installed families; a picked one that is gone stays. */
  function fontChoices(picked: string, own: string) {
    const families = picked && !fonts.includes(picked) ? [picked, ...fonts] : fonts;
    return [{ id: '', label: `${own} (Tend’s own)` }, ...families.map((f) => ({ id: f, label: f }))];
  }
</script>

<div class="top">
  <h2>Vault</h2>
  <p class="val mono">{vault.path ?? ''}</p>

  <h2>Sync</h2>
  {#if config}
    <div class="row"><span class="mono grow">{repo}</span><span class="meta">{config.branch}</span></div>
    <div class="row">
      <span class="grow">{syncLine}</span>
      <button class="link" disabled={sync.state === 'running'} onclick={sync.now}>Sync now</button>
    </div>
  {:else}
    <p class="val">Not connected. <a class="link" href={routes.settingsSection(SETTINGS_SECTION.sync)}>Connect…</a></p>
  {/if}

  <h2>Appearance</h2>
  <div class="p-seg wide" role="radiogroup" aria-label="Theme">
    {#each themeChoices as t (t.value)}
      <label class="p-choice" class:on={theme === t.value}>
        <input type="radio" name="theme" value={t.value} checked={theme === t.value} onchange={() => settings.set(settingKeys.theme, t.value)} />{t.label}
      </label>
    {/each}
  </div>
  <div class="row">
    <span class="grow">Menu button</span>
    <div class="p-seg" role="radiogroup" aria-label="Menu button">
      {#each sides as s (s.value)}
        <label class="p-choice" class:on={side === s.value}>
          <input type="radio" name="menu-side" value={s.value} checked={side === s.value} onchange={() => settings.set(settingKeys.menuSide, s.value)} />{s.label}
        </label>
      {/each}
    </div>
  </div>

  <h2>Fonts</h2>
  <div class="row">
    <span class="grow">App font</span>
    <ChoicePicker value={appFont} choices={fontChoices(appFont, 'Lato')} what="App font" field onpick={(id) => settings.set(settingKeys.appFont, id)} />
  </div>
  <div class="row">
    <span class="grow">Editor font</span>
    <ChoicePicker value={editorFont} choices={fontChoices(editorFont, 'Work Sans')} what="Editor font" field onpick={(id) => settings.set(settingKeys.editorFont, id)} />
  </div>
  <div class="row">
    <span class="grow">Text size</span>
    <div class="p-seg" role="radiogroup" aria-label="Text size">
      {#each textSizeChoices as t (t.value)}
        <label class="p-choice" class:on={textSize === t.value}>
          <input type="radio" name="text-size" value={t.value} checked={textSize === t.value} onchange={() => settings.set(settingKeys.textSize, t.value)} />{t.label}
        </label>
      {/each}
    </div>
  </div>

</div>

<style>
  .top {
    margin-bottom: 8px;
  }

  h2 {
    margin: 20px 0 4px;
    font: var(--text-section);
    font-size: 16px;
  }

  .val {
    margin: 8px 0;
    font: var(--text-ui);
    color: var(--text-muted);
    overflow-wrap: anywhere;
  }

  .mono {
    font: var(--text-mono);
  }

  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 44px;
    font: var(--text-ui);
    font-size: 15px;
    color: var(--text);
  }

  .grow {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .meta {
    font: var(--text-meta);
    color: var(--c-muted-2);
  }

  .link {
    padding: 0;
    border: 0;
    background: transparent;
    font: var(--text-ui);
    font-size: 15px;
    color: var(--primary);
    text-decoration: none;
  }

  .link:disabled {
    color: var(--c-muted-2);
  }

  .wide {
    margin: 8px 0;
  }

</style>
