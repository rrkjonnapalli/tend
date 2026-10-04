<script lang="ts">
  import type { Snippet } from 'svelte';
  import { page } from '$app/state';
  import Icon, { type IconName } from './Icon.svelte';
  import { ui } from '#lib/stores/ui.svelte.ts';

  let { right }: { right?: Snippet } = $props();

  const sections: { href: string; label: string; icon: IconName }[] = [
    { href: '/notes', label: 'Notes', icon: 'note' },
    { href: '/tasks', label: 'Tasks', icon: 'task' },
    { href: '/people', label: 'People', icon: 'person' },
  ];

  const isCurrent = (href: string) => page.url.pathname === href || page.url.pathname.startsWith(href + '/');
</script>

{#if ui.navOpen}
  <div class="bnav">
    <button class="tog" aria-label="Hide sections bar" title="Hide" onclick={ui.toggleNav}>
      <Icon name="chevronDown" size={12} stroke={2.5} />
    </button>
    <nav class="row" aria-label="Sections">
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
        {@render right?.()}
        <a
          class="ico"
          class:on={isCurrent('/settings')}
          href="/settings"
          aria-label="Settings"
          title="Settings">
          <Icon name="settings" size={16} />
        </a>
        <button
          class="ico"
          class:on={ui.popup === 'feedback'}
          aria-label="Feedback"
          title="Feedback"
          aria-haspopup="dialog"
          aria-expanded={ui.popup === 'feedback'}
          onclick={() => ui.toggle('feedback')}>
          <Icon name="feedback" size={16} />
        </button>
      </div>
    </nav>
  </div>
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
    position: relative;
    z-index: 1;
    width: 28px;
    height: 16px;
    margin-bottom: -8px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: var(--c-page);
    color: var(--c-faint);
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .tog:hover {
    color: var(--text);
    background: var(--c-hover);
  }

  .tog.closed {
    position: absolute;
    left: calc(50% - 14px);
    bottom: 6px;
    z-index: 6;
    margin: 0;
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
