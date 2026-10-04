// App-wide UI state that isn't tied to one screen.

export type Popup = 'feedback' | 'timers' | null;

class Ui {
  navOpen = $state(true);
  popup = $state<Popup>(null);

  toggleNav = () => {
    this.navOpen = !this.navOpen;
  };

  toggle = (p: Exclude<Popup, null>) => {
    this.popup = this.popup === p ? null : p;
  };
}

export const ui = new Ui();
