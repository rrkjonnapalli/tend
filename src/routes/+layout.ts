// Tauri has no Node server, so the app runs as a client-only SPA.
import { redirect } from '@sveltejs/kit';
import { vault } from '#lib/stores/vault.svelte.ts';

export const ssr = false;
export const prerender = false;

/** In a plain browser during development, answer commands from fixtures (see lib/dev). */
async function devMock() {
  if (import.meta.env.DEV && !('__TAURI_INTERNALS__' in window)) await import('#lib/dev/mock-ipc.ts');
}

// Runs before any page renders: reopen the last vault once, then send "/" (and every page
// when there's no vault) to the right place.
export async function load({ url }) {
  if (!vault.ready) {
    await devMock();
    await vault.restore();
  }
  if (!vault.path && url.pathname !== '/start') redirect(307, '/start');
  if (vault.path && url.pathname === '/') redirect(307, '/notes');
}
