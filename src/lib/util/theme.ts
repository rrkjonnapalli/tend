// Light or dark: the setting, or the OS when it's "system". The page reads `data-theme` on <html>
// (see the dark block in tokens.css).
import type { Theme, ThemePref } from '#lib/types/theme.type.ts';

const DARK_QUERY = '(prefers-color-scheme: dark)';

/** The theme to show for `pref`, given whether the OS is dark. */
export function resolveTheme(pref: ThemePref, systemDark: boolean): Theme {
  if (pref !== 'system') return pref;
  return systemDark ? 'dark' : 'light';
}

/** The theme a toggle switches to: the one not shown now. From "system" it becomes a fixed choice. */
export function toggledTheme(pref: ThemePref, systemDark: boolean): Theme {
  return resolveTheme(pref, systemDark) === 'dark' ? 'light' : 'dark';
}

/** Whether the OS is in its dark look right now. */
export const systemIsDark = () => window.matchMedia(DARK_QUERY).matches;

/** Shows the theme for `pref` and, for "system", follows the OS as it changes. Returns a stop. */
export function applyTheme(pref: ThemePref): () => void {
  const media = window.matchMedia(DARK_QUERY);
  const apply = () => {
    document.documentElement.dataset.theme = resolveTheme(pref, media.matches);
  };
  apply();
  if (pref !== 'system') return () => {};
  media.addEventListener('change', apply);
  return () => media.removeEventListener('change', apply);
}

/** The `data-surface` of something that stays light in the dark theme (a sticky note and the
 *  buttons on it): tokens.css gives it the light colour roles. None when it follows the theme. */
export const lightSurface = (stays: boolean) => (stays ? 'light' : undefined);
