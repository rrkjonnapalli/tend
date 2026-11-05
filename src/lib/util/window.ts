// The app window when running inside Tauri; a plain browser (dev fixtures) has none.
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { WindowAction } from '#lib/types/window.type.ts';

export const inTauri = () => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

/** Tend on a phone: no window to manage and no self-update (the stores update it). */
export const isMobileApp = typeof navigator !== 'undefined' && /Android|iPhone|iPad/.test(navigator.userAgent);

/** The desktop app, with its window and updater. */
export const inDesktopApp = () => inTauri() && !isMobileApp;

/** Closes, minimizes or maximizes the window; does nothing outside Tauri. */
export async function runWindowAction(action: WindowAction) {
  if (!inDesktopApp()) return;
  await getCurrentWindow()[action]();
}

/** Calls `onchange` now and on every resize with whether the window fills the screen. Returns a stop function. */
export function watchFilled(onchange: (filled: boolean) => void) {
  if (!inDesktopApp()) return () => {};
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
