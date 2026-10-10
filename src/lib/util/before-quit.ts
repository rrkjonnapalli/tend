// Work that must finish before the app quits (saving an open note). Pages register it; the layout
// runs it all when Rust says the app is quitting (`app:quitting`) and then answers `app_flushed`.

import type { QuitHook } from '#lib/types/quit.type.ts';

const hooks = new Set<QuitHook>();

/** Runs `hook` before the app quits, while the caller is mounted. Returns the unregister. */
export function onBeforeQuit(hook: QuitHook): () => void {
  hooks.add(hook);
  return () => hooks.delete(hook);
}

/** Runs every hook; one that fails doesn't stop the others. */
export async function runBeforeQuit(): Promise<void> {
  // async, so a hook that throws at once is caught like one that rejects.
  await Promise.allSettled([...hooks].map(async (hook) => hook()));
}
