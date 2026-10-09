// A short status line at the bottom of the window ("Renamed to …"). Errors get their own look,
// stay longer, are announced at once, and close on click.

const SHOW_MS = 2200;
const ERROR_MS = 8000;

class Toast {
  text = $state('');
  isError = $state(false);
  #timer: ReturnType<typeof setTimeout> | undefined;

  show = (text: string) => {
    this.#put(text, { isError: false, ms: SHOW_MS });
  };

  error = (text: string) => {
    this.#put(text, { isError: true, ms: ERROR_MS });
  };

  hide = () => {
    clearTimeout(this.#timer);
    this.text = '';
  };

  #put(text: string, { isError, ms }: { isError: boolean; ms: number }) {
    this.text = text;
    this.isError = isError;
    clearTimeout(this.#timer);
    this.#timer = setTimeout(this.hide, ms);
  }
}

export const toast = new Toast();
