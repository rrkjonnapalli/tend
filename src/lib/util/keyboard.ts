// The phone's on-screen keyboard. Android draws apps edge to edge, so the WebView is never resized
// for it: only the visual viewport shrinks. `--kb` on the root holds the keyboard's height, and the
// phone layout lifts its bottom bars and sheets by it (tend-mobile-spec.md §4).

/** The part of the window the keyboard covers, in px; 0 when it is down. */
export const keyboardHeight = ({ window, visible }: { window: number; visible: number }) => Math.max(0, Math.round(window - visible));

/** The height popovers and sheets can use: above the keyboard. */
export const visibleHeight = () => window.visualViewport?.height ?? window.innerHeight;

/** Keeps `--kb` up to date while the phone layout shows; returns the stop function. */
export function watchKeyboard(): () => void {
  const vv = window.visualViewport;
  if (!vv) return () => {};
  const root = document.documentElement.style;
  const update = () => root.setProperty('--kb', `${keyboardHeight({ window: window.innerHeight, visible: vv.height })}px`);
  update();
  vv.addEventListener('resize', update);
  return () => {
    vv.removeEventListener('resize', update);
    root.removeProperty('--kb');
  };
}
