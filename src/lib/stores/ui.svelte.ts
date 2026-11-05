// App-wide UI state that isn't tied to one screen.
import { settings } from './settings.svelte.ts';
import type { Popup, QuickKind } from '#lib/types/ui.type.ts';
import { settingDefaults, settingKeys } from '#lib/config/settings.config.ts';
import { config } from '#lib/config/app.config.ts';
import { MediaQuery } from 'svelte/reactivity';

const phoneQuery = new MediaQuery(`max-width: ${config.phoneMaxWidth}px`);

class Ui {
  popup = $state<Popup>(null);
  palette = $state(false);
  quick = $state<QuickKind | null>(null);

  /** The phone layout: a narrow window (always on a phone). */
  get phone(): boolean {
    return phoneQuery.current;
  }

  get navOpen(): boolean {
    return settings.get(settingKeys.navOpen, settingDefaults.navOpen);
  }

  toggleNav = () => {
    settings.set(settingKeys.navOpen, !this.navOpen);
  };

  get explorerOpen(): boolean {
    return settings.get(settingKeys.explorerOpen, settingDefaults.explorerOpen);
  }

  toggleExplorer = () => {
    settings.set(settingKeys.explorerOpen, !this.explorerOpen);
  };

  toggle = (p: Exclude<Popup, null>) => {
    this.popup = this.popup === p ? null : p;
  };

  /** Opens a bottom-bar popover, showing the bar first if it's hidden. */
  show = (p: Exclude<Popup, null>) => {
    if (!this.navOpen) this.toggleNav();
    this.popup = p;
  };

  closePopup = () => {
    this.popup = null;
  };
}

export const ui = new Ui();
