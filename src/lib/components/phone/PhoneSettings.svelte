<script lang="ts">
  // Settings on a phone (MD-Settings): the vault, sync, appearance (theme and the menu button's
  // side) and fonts on one page, then rows that open the other sections. Spec: tend-mobile-spec.md §11.
  import { onMount } from 'svelte';
  import Icon from '../Icon.svelte';
  import { appAPI } from '#lib/api/app.ts';
  import { vault } from '#lib/stores/vault.svelte.ts';
  import { sync } from '#lib/stores/sync.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { settingDefaults, settingKeys, textSizeChoices, themeChoices } from '#lib/config/settings.config.ts';
  import { SETTINGS_SECTION, type SettingsSection } from '#lib/enums/settings.enum.ts';
  import type { ThemePref } from '#lib/types/theme.type.ts';
  import type { TextSize } from '#lib/types/font.type.ts';
  import type { MenuSide } from '#lib/types/ui.type.ts';
  import { ago } from '#lib/util/dates.ts';
  import { routes } from '#lib/util/routes.ts';

  const sections: { id: SettingsSection; label: string; sub: string }[] = [
    { id: SETTINGS_SECTION.general, label: 'General', sub: 'files, confirmations' },
    { id: SETTINGS_SECTION.tasks, label: 'Tasks', sub: 'statuses, labels' },
    { id: SETTINGS_SECTION.library, label: 'Library', sub: 'dictionary, kinds' },
    { id: SETTINGS_SECTION.tags, label: 'Tags', sub: 'active, blocked' },
    { id: SETTINGS_SECTION.sync, label: 'Sync', sub: 'repo, token, schedule' },
  ];
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
  const repo = $derived(config?.url.replace(/^https?:\/\//, '').replace(/\.git$/, '') ?? '');
  const lastOk = $derived(sync.status?.lastOkAt ?? null);

  /** The font list: Tend's own first, then the installed families; a picked one that is gone stays. */
  function familiesWith(picked: string) {
    return picked && !fonts.includes(picked) ? [picked, ...fonts] : fonts;
  }
</script>

<main class="phone-list">
  <h1>Settings</h1>

  <h2>Vault</h2>
  <p class="val mono">{vault.path ?? ''}</p>

  <h2>Sync</h2>
  {#if config}
    <div class="row"><span class="mono grow">{repo}</span><span class="meta">{config.branch}</span></div>
    <div class="row">
      <span class="grow">{sync.state === 'running' ? 'Syncing…' : lastOk ? `Synced ${ago(lastOk)}` : 'Not synced yet'}</span>
      <button class="link" disabled={sync.state === 'running'} onclick={sync.now}>Sync now</button>
    </div>
  {:else}
    <p class="val">Not connected. <a class="link" href={routes.settingsSection(SETTINGS_SECTION.sync)}>Connect…</a></p>
  {/if}

  <h2>Appearance</h2>
  <div class="seg wide" role="radiogroup" aria-label="Theme">
    {#each themeChoices as t (t.value)}
      <label class="choice" class:on={theme === t.value}>
        <input type="radio" name="theme" value={t.value} checked={theme === t.value} onchange={() => settings.set(settingKeys.theme, t.value)} />{t.label}
      </label>
    {/each}
  </div>
  <div class="row">
    <span class="grow">Menu button</span>
    <div class="seg" role="radiogroup" aria-label="Menu button">
      {#each sides as s (s.value)}
        <label class="choice" class:on={side === s.value}>
          <input type="radio" name="menu-side" value={s.value} checked={side === s.value} onchange={() => settings.set(settingKeys.menuSide, s.value)} />{s.label}
        </label>
      {/each}
    </div>
  </div>

  <h2>Fonts</h2>
  <label class="row">
    <span class="grow">App font</span>
    <select class="pick" value={appFont} onchange={(e) => settings.set(settingKeys.appFont, e.currentTarget.value)}>
      <option value="">Lato (Tend’s own)</option>
      {#each familiesWith(appFont) as f (f)}<option value={f}>{f}</option>{/each}
    </select>
  </label>
  <label class="row">
    <span class="grow">Editor font</span>
    <select class="pick" value={editorFont} onchange={(e) => settings.set(settingKeys.editorFont, e.currentTarget.value)}>
      <option value="">Work Sans (Tend’s own)</option>
      {#each familiesWith(editorFont) as f (f)}<option value={f}>{f}</option>{/each}
    </select>
  </label>
  <div class="row">
    <span class="grow">Text size</span>
    <div class="seg" role="radiogroup" aria-label="Text size">
      {#each textSizeChoices as t (t.value)}
        <label class="choice" class:on={textSize === t.value}>
          <input type="radio" name="text-size" value={t.value} checked={textSize === t.value} onchange={() => settings.set(settingKeys.textSize, t.value)} />{t.label}
        </label>
      {/each}
    </div>
  </div>

  <h2>Sections</h2>
  {#each sections as s (s.id)}
    <a class="qr" href={routes.settingsSection(s.id)}>
      <span class="tt">{s.label}</span><span class="meta">{s.sub}</span><Icon name="chevronRight" size={14} />
    </a>
  {/each}
</main>

<style>
  h1 {
    margin: 24px 0 8px;
    font: var(--text-heading);
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

  .seg {
    display: flex;
    padding: 3px;
    border-radius: var(--radius-md);
    background: var(--c-chip);
  }

  .seg.wide {
    margin: 8px 0;
  }

  .choice {
    flex: 1;
    display: grid;
    place-items: center;
    min-width: 56px;
    height: 36px;
    padding: 0 10px;
    border-radius: var(--radius-sm);
    font: var(--text-ui);
    color: var(--c-muted-2);
  }

  .choice.on {
    background: var(--c-page);
    color: var(--text);
    font-weight: 700;
  }

  .choice input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .choice:focus-within {
    background: var(--c-hover);
  }

  .pick {
    max-width: 55%;
    height: 36px;
    padding: 0 8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: var(--c-chip);
    font: var(--text-ui);
    color: var(--text);
  }

  .qr :global(svg) {
    flex: none;
    color: var(--c-faint);
  }
</style>
