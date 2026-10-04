// Running timers. The countdown is `endsAt − now`, recomputed every second (never counted
// down), so it stays right after sleep or throttling. A timer rings once `now ≥ endsAt`.
import { timersApi, type Timer } from '#lib/api/timers.ts';
import { messageOf } from '#lib/api/core.ts';
import { chime, notify } from '#lib/util/notify.ts';
import { settings } from './settings.svelte.ts';
import { toast } from './toast.svelte.ts';

const TICK_MS = 1000;

class TimersStore {
  list = $state<Timer[]>([]);
  now = $state(Date.now());
  #ticking = false;

  /** Loads running timers and reports the ones that ended while the app was closed. */
  start = async () => {
    const missed = await timersApi.finishedWhileClosed();
    if (missed.length) {
      const names = missed.map((t) => t.title || 'Timer').join(', ');
      toast.show(`${names} finished while Tend was closed`);
    }
    this.list = await timersApi.list();
    if (this.#ticking) return;
    this.#ticking = true;
    setInterval(this.tick, TICK_MS);
    // After sleep the next interval can be late; waking the window checks at once.
    document.addEventListener('visibilitychange', this.tick);
    window.addEventListener('focus', this.tick);
  };

  tick = () => {
    this.now = Date.now();
    for (const t of this.list.filter((t) => t.endsAt <= this.now)) this.#ring(t);
  };

  #ring = async (t: Timer) => {
    this.list = this.list.filter((x) => x.id !== t.id);
    await timersApi.ring(t.id);
    if (settings.get('timers.sound', true)) chime();
    await notify(t.title || 'Timer', 'Time’s up').catch(() => toast.show(`${t.title || 'Timer'}: time’s up`));
  };

  add = async (title: string, ms: number) => {
    try {
      const t = await timersApi.start(title, ms);
      this.list = [...this.list, t].sort((a, b) => a.endsAt - b.endsAt);
      return t;
    } catch (e) {
      toast.show(messageOf(e));
    }
  };

  addMinute = async (id: string) => {
    await timersApi.addMinute(id);
    this.list = await timersApi.list();
  };

  cancel = async (id: string) => {
    await timersApi.cancel(id);
    this.list = this.list.filter((t) => t.id !== id);
  };
}

export const timers = new TimersStore();
