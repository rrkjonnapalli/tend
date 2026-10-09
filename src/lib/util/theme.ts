// Light or dark: the setting, or the OS when it's "system". The page reads `data-theme` on <html>
// (see the dark block in tokens.css).
import type { Theme, ThemePref } from '#lib/types/theme.type.ts';

const DARK_QUERY = '(prefers-color-scheme: dark)';

/** The theme to show for `pref`, given whether the OS is dark. */
export function resolveTheme(pref: ThemePref, systemDark: boolean): Theme {
  if (pref !== 'system') return pref;
  return systemDark ? 'dark' : 'light';
}

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
