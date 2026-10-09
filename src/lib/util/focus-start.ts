// Where Tab starts after a navigation. SvelteKit resets it to <body>, so the next Tab reached the
// window's close and minimise buttons first; the page content is where Tab should begin.

/** Makes `el` the starting point for Tab without leaving it focused or showing a ring. */
export function startTabbingIn(el: HTMLElement) {
  const had = el.getAttribute('tabindex');
  el.setAttribute('tabindex', '-1');
  el.focus({ preventScroll: true, focusVisible: false } as FocusOptions);
  if (had === null) el.removeAttribute('tabindex');
  else el.setAttribute('tabindex', had);
}
