<script lang="ts">
  // The sections bar at the bottom: Notes, Tasks, People, plus timers, sync, settings and feedback.
  import type { IconName } from '#lib/types/icon.type.ts';
  import { page } from '$app/state';
  import Icon from './Icon.svelte';
  import FeedbackPopover from './FeedbackPopover.svelte';
  import TimerChip from './TimerChip.svelte';
  import SyncIcon from './SyncIcon.svelte';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { zoneNav } from '#lib/actions/zone-nav.action.ts';
  import { routes } from '#lib/util/routes.ts';

  const sections: { href: string; label: string; icon: IconName }[] = [
    { href: routes.notes, label: 'Notes', icon: 'note' },
    { href: routes.tasks, label: 'Tasks', icon: 'task' },
    { href: routes.people, label: 'People', icon: 'person' },
  ];

  let feedbackBtn = $state<HTMLButtonElement>();

  const feedbackOpen = $derived(ui.popup === 'feedback');

  /** The section's own page or any page inside it. */
  function isCurrent(href: string) {
    const path = page.url.pathname;
    return path === href || path.startsWith(`${href}/`);
  }
</script>

{#if ui.navOpen}
  <div class="bnav">
    <button class="tog" aria-label="Hide sections bar" title="Hide" onclick={ui.toggleNav}>
      <Icon name="chevronDown" size={12} stroke={2.5} />
    </button>
    <nav class="row" aria-label="Sections" data-pane="dock" data-zone="bar" use:zoneNav>
      {#each sections as s (s.href)}
        <a
          class="ico"
          class:cur={isCurrent(s.href)}
          href={s.href}
          aria-label={s.label}
          title={s.label}
          aria-current={isCurrent(s.href) ? 'page' : undefined}>
          <Icon name={s.icon} size={18} />
        </a>
      {/each}
      <div class="right">
        <TimerChip />
        <SyncIcon />
        <a
          class="ico"
          class:cur={isCurrent(routes.settings)}
          aria-current={isCurrent(routes.settings) ? 'page' : undefined}
          href={routes.settings}
          aria-label="Settings"
          title="Settings">
          <Icon name="settings" size={16} />
        </a>
        <button
          bind:this={feedbackBtn}
          class="ico"
          class:on={feedbackOpen}
          aria-label="Feedback"
          title="Feedback"
          aria-haspopup="dialog"
          aria-expanded={feedbackOpen}
          onclick={() => ui.toggle('feedback')}>
          <Icon name="feedback" size={16} />
        </button>
      </div>
    </nav>
  </div>
  {#if feedbackOpen}<FeedbackPopover anchor={feedbackBtn} onclose={ui.closePopup} />{/if}
{:else}
  <button class="tog closed" aria-label="Show sections bar" title="Show" onclick={ui.toggleNav}>
    <Icon name="chevronUp" size={12} stroke={2.5} />
  </button>
{/if}

<style>
  .bnav {
    flex: 0 0 auto;
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .tog {
    position: absolute;
    top: -8px;
    left: calc(50% - 14px);
    z-index: 1;
    width: 28px;
    height: 16px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: var(--c-page);
    color: var(--c-muted-2);
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .tog:hover {
    color: var(--text);
    background: var(--c-hover);
  }

  .tog.closed {
    top: auto;
    bottom: 6px;
    z-index: 6;
    background: transparent;
  }

  .tog.closed:hover {
    background: var(--c-hover);
  }

  .row {
    position: relative;
    align-self: stretch;
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 14px;
    padding: 8px 10px 10px;
    background: var(--c-page);
    border-top: 1px solid var(--border);
  }

  .ico {
    position: relative;
    display: grid;
    place-items: center;
    width: 28px;
    height: 26px;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--c-muted-2);
    text-decoration: none;
    cursor: pointer;
  }

  .ico:hover,
  .ico.on {
    color: var(--text);
  }

  .ico.cur {
    color: var(--c-nav-active);
  }

  .ico.cur::after {
    content: '';
    position: absolute;
    bottom: -6px;
    left: 50%;
    width: 4px;
    height: 4px;
    margin-left: -2px;
    border-radius: 2px;
    background: var(--c-nav-active);
  }

  .right {
    position: absolute;
    right: 16px;
    top: 8px;
    display: flex;
    align-items: center;
    gap: 4px;
  }
</style>
