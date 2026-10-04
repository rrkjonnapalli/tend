// App-wide UI state that isn't tied to one screen.
import { settings } from './settings.svelte.ts';
import type { Popup, QuickKind } from '#lib/types/ui.type.ts';


class Ui {
  popup = $state<Popup>(null);
  palette = $state(false);
  quick = $state<QuickKind | null>(null);

  get navOpen(): boolean {
    return settings.get('ui.nav_open', true);
  }

  toggleNav = () => {
    settings.set('ui.nav_open', !this.navOpen);
  };

  get explorerOpen(): boolean {
    return settings.get('ui.explorer_open', true);
  }

  toggleExplorer = () => {
    settings.set('ui.explorer_open', !this.explorerOpen);
  };

  toggle = (p: Exclude<Popup, null>) => {
    this.popup = this.popup === p ? null : p;
  };

  /** Opens a bottom-bar popover, showing the bar first if it's hidden. */
  show = (p: Exclude<Popup, null>) => {
    if (!this.navOpen) this.toggleNav();
    this.popup = p;
  };
}

export const ui = new Ui();
