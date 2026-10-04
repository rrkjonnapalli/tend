// Tauri has no Node server, so the app runs as a client-only SPA.
import { vault } from '#lib/stores/vault.svelte.ts';

export const ssr = false;
export const prerender = false;

/** In a plain browser during development, answer commands from fixtures (see lib/dev). */
async function devMock() {
  if (import.meta.env.DEV && !('__TAURI_INTERNALS__' in window)) await import('#lib/dev/mock-ipc.ts');
}

let started = false;

// Starts reopening the last vault without waiting for it, so the layout can show
// "Opening vault…" (or an error) instead of a blank window. The layout redirects once it's done.
export function load() {
  if (started) return;
  started = true;
  devMock().then(vault.restore);
}
