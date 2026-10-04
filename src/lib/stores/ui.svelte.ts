// App-wide UI state that isn't tied to one screen.
import { settings } from './settings.svelte.ts';

export type Popup = 'feedback' | 'timers' | null;

class Ui {
  popup = $state<Popup>(null);

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
}

export const ui = new Ui();
