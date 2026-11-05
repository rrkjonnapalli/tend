<script lang="ts">
  // The phone's only navigation: a round button tucked into a bottom corner that grows into the
  // menu (Notes · Tasks · People · Library, then Sync and Settings). It hides while a field has
  // focus, since the keyboard is up then. Spec: docs/mobile/tend-mobile-spec.md §4.
  import type { IconName } from '#lib/types/icon.type.ts';
  import type { MenuSide } from '#lib/types/ui.type.ts';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import Icon from '../Icon.svelte';
  import TokenSheet from './TokenSheet.svelte';
  import { sync } from '#lib/stores/sync.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { settingDefaults, settingKeys } from '#lib/config/settings.config.ts';
  import { SETTINGS_SECTION } from '#lib/enums/settings.enum.ts';
  import { routes } from '#lib/util/routes.ts';
  import { isTypingTarget } from '#lib/util/keys.ts';
  import { timeLabel } from '#lib/util/dates.ts';

  const sections: { href: string; label: string; icon: IconName }[] = [
    { href: routes.notes, label: 'Notes', icon: 'note' },
    { href: routes.tasks, label: 'Tasks', icon: 'task' },
    { href: routes.people, label: 'People', icon: 'person' },
    { href: routes.library, label: 'Library', icon: 'library' },
  ];

  let open = $state(false);
  let typing = $state(false);
  let tokenOpen = $state(false);

  const side = $derived(settings.get<MenuSide>(settingKeys.menuSide, settingDefaults.menuSide));
  const current = $derived(sections.find((s) => isCurrent(s.href)));
  const icon = $derived<IconName>(current?.icon ?? 'settings');
  /** The sync dot and icon colour: running, failed, or waiting for a new token. */
  const syncMark = $derived(markOf());
  const lastSync = $derived(sync.status?.lastOkAt ? clock(sync.status.lastOkAt) : '');

  function isCurrent(href: string) {
    const path = page.url.pathname;
    return path === href || path.startsWith(`${href}/`);
  }

  function markOf(): 'run' | 'fail' | 'tok' | '' {
    if (sync.state === 'running') return 'run';
    if (sync.needsToken) return 'tok';
    if (sync.state === 'failed') return 'fail';
    return '';
  }

  function clock(ms: number) {
    const d = new Date(ms);
    const hhmm = `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
    return timeLabel(hhmm);
  }

  /** Idle: sync now. Refused token: the Replace token sheet. Failed: Settings → Sync, which shows
   *  why. Running: nothing. */
  function onSync() {
    open = false;
    if (syncMark === 'run') return;
    if (syncMark === 'tok') tokenOpen = true;
    else if (syncMark === 'fail') goto(routes.settingsSection(SETTINGS_SECTION.sync));
    else sync.now();
  }

  function onkeydown(e: KeyboardEvent) {
    if (open && e.key === 'Escape') open = false;
  }

  function onfocusin(e: FocusEvent) {
    typing = isTypingTarget(e.target);
  }

  function onfocusout() {
    typing = false;
  }
</script>

<svelte:window {onkeydown} {onfocusin} {onfocusout} />

{#if !typing}
  <button class="veil" class:open tabindex="-1" aria-hidden="true" onclick={() => (open = false)}></button>
  <nav class="cm" class:open class:l={side === 'left'} aria-label="Menu">
    <div class="items" inert={!open}>
      {#each sections as s (s.href)}
        <a class="r" class:cur={current === s} href={s.href} aria-current={current === s ? 'page' : undefined} onclick={() => (open = false)}>
          <span class="i"><Icon name={s.icon} size={18} /></span>{s.label}
        </a>
      {/each}
    </div>
    <div class="brow">
      <span class="tools" inert={!open}>
        {#if sync.state !== 'off'}
          <button class="t {syncMark}" aria-label={lastSync ? `Sync now, last synced ${lastSync}` : 'Sync now'} onclick={onSync}>
            <Icon name="sync" size={18} />
            {#if lastSync}<span class="m">{lastSync}</span>{/if}
          </button>
        {/if}
        <a class="t" class:cur={isCurrent(routes.settings)} href={routes.settings} aria-label="Settings" onclick={() => (open = false)}>
          <Icon name="settings" size={18} />
        </a>
      </span>
      <button class="btn" aria-label={open ? 'Close menu' : `Menu: ${current?.label ?? 'Settings'}`} aria-expanded={open} onclick={() => (open = !open)}>
        <span class="a"><Icon name={icon} size={22} /></span>
        <span class="b"><Icon name="close" size={22} /></span>
        {#if syncMark}<span class="dot {syncMark}" aria-hidden="true"></span>{/if}
      </button>
    </div>
  </nav>
{/if}
{#if tokenOpen}<TokenSheet onclose={() => (tokenOpen = false)} />{/if}

<style>
  /* Sizes and timing from docs/mobile/design/tend-mobile2.css (.cm). */
  .veil {
    position: fixed;
    inset: 0;
    z-index: 40;
    padding: 0;
    border: 0;
    background: rgba(37, 44, 51, 0.08);
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.25s;
  }

  .veil.open {
    opacity: 1;
    pointer-events: auto;
  }

  .cm {
    position: fixed;
    right: -18px;
    bottom: calc(40px + env(safe-area-inset-bottom));
    z-index: 41;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    align-items: flex-end;
    width: 60px;
    height: 56px;
    padding-right: 18px;
    border-radius: 28px;
    background: var(--c-page);
    box-shadow:
      0 0 0 1px var(--border),
      0 12px 32px -14px rgba(37, 44, 51, 0.32);
    overflow: hidden;
    transition:
      width 0.3s cubic-bezier(0.2, 0.8, 0.2, 1),
      height 0.3s cubic-bezier(0.2, 0.8, 0.2, 1),
      border-radius 0.3s;
  }

  .cm.l {
    right: auto;
    left: -18px;
    padding-right: 0;
    padding-left: 18px;
    align-items: flex-start;
  }

  .cm.open {
    width: 236px;
    height: 244px;
    border-radius: 24px;
  }

  .items {
    flex: none;
    width: 218px;
    padding: 6px 6px 0;
    opacity: 0;
    transform: translateY(10px);
    transition:
      opacity 0.15s,
      transform 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
  }

  .cm.open .items {
    opacity: 1;
    transform: none;
    transition-delay: 0.08s, 0s;
  }

  .r {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    height: 44px;
    padding: 0 12px;
    border-radius: 16px;
    font: var(--text-ui);
    font-size: 15px;
    color: var(--text);
    text-decoration: none;
    white-space: nowrap;
  }

  .r .i {
    display: grid;
    color: var(--c-muted-2);
  }

  .r:hover,
  .r:focus-visible {
    background: var(--c-hover);
  }

  .r.cur {
    background: var(--c-selected);
    color: var(--c-current-ink);
  }

  .r.cur .i {
    color: var(--c-current);
  }

  .brow {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 218px;
    height: 56px;
    padding: 0 0 0 6px;
  }

  /* Mirrored: × at the left edge, Sync at the outer (right) end. */
  .cm.l .brow {
    flex-direction: row-reverse;
    padding: 0 6px 0 0;
  }

  .cm.l .tools {
    flex-direction: row-reverse;
  }

  .tools {
    display: flex;
    align-items: center;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .cm.open .tools {
    opacity: 1;
    transition-delay: 0.1s;
  }

  .t {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 44px;
    min-width: 44px;
    padding: 0 10px;
    border: 0;
    border-radius: 16px;
    background: transparent;
    color: var(--c-muted-2);
    text-decoration: none;
  }

  .t:hover,
  .t:focus-visible {
    background: var(--c-hover);
  }

  .t.cur {
    color: var(--c-current);
  }

  .t .m {
    font: var(--text-meta);
  }

  .t.run {
    color: var(--c-current);
  }

  .t.fail {
    color: var(--danger);
  }

  .t.tok {
    color: var(--warning);
  }

  .btn {
    position: relative;
    flex: none;
    width: 42px;
    height: 56px;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--c-nav-active);
    cursor: pointer;
  }

  .a,
  .b {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    transition:
      opacity 0.2s,
      transform 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
  }

  .b {
    opacity: 0;
    transform: rotate(-90deg);
    color: var(--text-muted);
  }

  .cm.open .a {
    opacity: 0;
    transform: rotate(90deg);
  }

  .cm.open .b {
    opacity: 1;
    transform: none;
  }

  .dot {
    position: absolute;
    top: 11px;
    right: 4px;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    box-shadow: 0 0 0 2px var(--c-page);
  }

  .cm.l .dot {
    right: auto;
    left: 30px;
  }

  .cm.open .dot {
    opacity: 0;
  }

  .dot.run {
    background: var(--c-current);
  }

  .dot.fail {
    background: var(--danger);
  }

  .dot.tok {
    background: var(--warning);
  }

  @media (prefers-reduced-motion: reduce) {
    .cm,
    .items,
    .tools,
    .a,
    .b,
    .veil {
      transition: none;
    }
  }
</style>
