// A short status line at the bottom of the window ("Renamed to …").

const SHOW_MS = 2200;

class Toast {
  text = $state('');
  #timer: ReturnType<typeof setTimeout> | undefined;

  show = (text: string) => {
    this.text = text;
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => (this.text = ''), SHOW_MS);
  };
}

export const toast = new Toast();
