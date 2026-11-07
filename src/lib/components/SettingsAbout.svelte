<script lang="ts">
  // Settings → About: the running version, the fonts and libraries Tend is built on, and their
  // licence notices (static/third-party-licenses.txt, written by `bun run notices`).
  import { getVersion } from '@tauri-apps/api/app';
  import { onMount } from 'svelte';
  import { config } from '#lib/config/app.config.ts';
  import { AppError } from '#lib/errors/app-error.ts';
  import { errors } from '#lib/errors/registry.ts';
  import { inTauri } from '#lib/util/window.ts';

  const FONTS = [
    { name: 'Montserrat, Lato, JetBrains Mono, Work Sans', note: 'the app and editor fonts, SIL Open Font License 1.1' },
    { name: 'Caveat, Gaegu', note: 'the handwriting on snaps, SIL Open Font License 1.1' },
    { name: 'Smile Moon', note: 'stack names, by MJType, all rights reserved' },
  ];

  let version = $state('');
  let notices = $state<string | null>(null);
  let loading = $state(false);

  onMount(async () => {
    if (inTauri()) version = await getVersion();
  });

  async function showNotices() {
    loading = true;
    try {
      const res = await fetch(config.noticesPath);
      if (!res.ok) throw new AppError(errors.NOT_FOUND, 'The licence notices are missing from this build.');
      notices = await res.text();
    } finally {
      loading = false;
    }
  }
</script>

<section class="group">
  <h2 class="section-title">Tend</h2>
  <p class="line">Version {version || '—'}</p>
</section>

<section class="group">
  <h2 class="section-title">Fonts</h2>
  <ul class="credits">
    {#each FONTS as f (f.name)}
      <li><span class="name">{f.name}</span> <span class="note">· {f.note}</span></li>
    {/each}
  </ul>
</section>

<section class="group">
  <h2 class="section-title">Licences</h2>
  <p class="hint">Tend is built with Tauri, SvelteKit, CodeMirror, marked, DOMPurify, libgit2, OpenSSL, SQLite and many other open-source libraries. Their licences ask for these notices to go along with the app.</p>
  {#if notices === null}
    <div><button class="link-btn" disabled={loading} onclick={showNotices}>{loading ? 'Loading…' : 'Show the licence notices'}</button></div>
  {:else}
    <pre class="notices" tabindex="-1">{notices}</pre>
  {/if}
</section>

<style>
  .group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .line {
    margin: 0;
    font: var(--text-ui);
    color: var(--text);
  }

  .credits {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 6px;
    font: var(--text-ui);
  }

  .note,
  .hint {
    color: var(--c-muted-2);
  }

  .hint {
    margin: 0;
    font: var(--text-meta);
  }

  .notices {
    max-height: 420px;
    margin: 0;
    padding: 12px;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    font: var(--text-mono);
    font-size: 11px;
    line-height: 16px;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    color: var(--text-muted);
  }
</style>
