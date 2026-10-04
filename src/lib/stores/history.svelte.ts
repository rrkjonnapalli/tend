// In-app history, so Back returns to the previous page (like a browser) and falls back to a
// parent page only when there is nothing to go back to (e.g. right after launch).
import { goto, type AfterNavigate } from '$app/navigation';

class History {
  #stack: string[] = [];
  #index = -1;
  /** Set by `replace()`, read by the next `record()`. */
  #replacing = false;

  /** Call from the root layout's afterNavigate. */
  record = (nav: AfterNavigate) => {
    const url = nav.to ? nav.to.url.pathname + nav.to.url.search : '';
    if (nav.type === 'enter') this.#start(url);
    else if (nav.type === 'popstate') this.#step(nav.delta ?? 0);
    else if (this.#replacing) this.#stack[this.#index] = url;
    else this.#push(url);
    this.#replacing = false;
  };

  /** Navigates without adding a step (redirects, filters, renamed folders). */
  replace = (url: string) => {
    this.#replacing = true;
    return goto(url, { replace: true });
  };

  /** The previous page, or `fallback` when this is the first page of the session. */
  back = (fallback: string) => {
    if (this.#index > 0) history.back();
    else this.replace(fallback);
  };

  #start(url: string) {
    this.#stack = [url];
    this.#index = 0;
  }

  /** Browser back/forward: move by `delta`, staying inside the stack. */
  #step(delta: number) {
    const last = this.#stack.length - 1;
    const target = this.#index + delta;
    this.#index = Math.max(0, Math.min(last, target));
  }

  /** A new page drops any forward steps, like a browser. */
  #push(url: string) {
    const kept = this.#stack.slice(0, this.#index + 1);
    this.#stack = [...kept, url];
    this.#index += 1;
  }
}

export const nav = new History();
