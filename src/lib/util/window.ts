// The app window when running inside Tauri; a plain browser (dev fixtures) has none.
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { WindowAction } from '#lib/types/window.type.ts';

export const inTauri = () => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

/** Closes, minimizes or maximizes the window; does nothing outside Tauri. */
export async function runWindowAction(action: WindowAction) {
  if (!inTauri()) return;
  await getCurrentWindow()[action]();
}

/** Calls `onchange` now and on every resize with whether the window fills the screen. Returns a stop function. */
export function watchFilled(onchange: (filled: boolean) => void) {
  if (!inTauri()) return () => {};
  const win = getCurrentWindow();
  const check = async () => {
    const filled = (await win.isMaximized()) || (await win.isFullscreen());
    onchange(filled);
  };
  check();
  const unlisten = win.onResized(check);
  return () => {
    unlisten.then((stop) => stop());
  };
}
