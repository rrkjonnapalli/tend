// ↑↓ Home End through the choices of an open dropdown (a menu, a list of presets): every
// `role="option"`, `role="menuitem"` or `data-option` control inside `node`. The keys are heard on
// the window, since focus starts on the button that opened the dropdown; Enter and Space then
// press the focused choice as they press any button. A dropdown that moves its own highlight (a
// search field over a list, a trigger that keeps focus) handles the key first and is left alone.
import { isTypingTarget } from '#lib/util/keys.ts';
import { moveTo } from '#lib/util/list-nav.ts';

const OPTION = '[role="option"], [role="menuitem"], [data-option]';

export function optionNav(node: HTMLElement) {
  function onkeydown(e: KeyboardEvent) {
    const hasModifier = e.metaKey || e.ctrlKey || e.altKey || e.shiftKey;
    if (e.defaultPrevented || hasModifier || isTypingTarget(e.target)) return;
    const options = [...node.querySelectorAll<HTMLElement>(OPTION)];
    const at = options.indexOf(document.activeElement as HTMLElement);
    const next = moveTo({ key: e.key, at, count: options.length });
    if (next === null) return;
    e.preventDefault();
    options[next].focus();
    options[next].scrollIntoView({ block: 'nearest' });
  }

  window.addEventListener('keydown', onkeydown);
  return {
    destroy() {
      window.removeEventListener('keydown', onkeydown);
    },
  };
}
