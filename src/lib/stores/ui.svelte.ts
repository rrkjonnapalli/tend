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

  toggle = (p: Exclude<Popup, null>) => {
    this.popup = this.popup === p ? null : p;
  };
}

export const ui = new Ui();
