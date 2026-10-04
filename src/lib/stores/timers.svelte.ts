// Running timers. The countdown is `endsAt − now`, recomputed every second (never counted
// down), so it stays right after sleep or throttling. A timer rings once `now ≥ endsAt`.
// The clock only ticks while there are timers to count down.
import { timersAPI } from '#lib/api/timers.ts';
import { messageOf } from '#lib/api/core.ts';
import type { Timer } from '#lib/types/timer.type.ts';
import { allowNotifications, chime, notify } from '#lib/util/notify.ts';
import { timerName } from '#lib/util/timer-text.ts';
import { settings } from './settings.svelte.ts';
import { toast } from './toast.svelte.ts';
import { settingKeys } from '#lib/config/settings.config.ts';

const TICK_MS = 1000;

const bySoonest = (a: Timer, b: Timer) => a.endsAt - b.endsAt;

class TimersStore {
  list = $state<Timer[]>([]);
  now = $state(Date.now());
  /** True between `start()` and `stop()`, i.e. while a vault is open. */
  #running = false;
  #interval: ReturnType<typeof setInterval> | undefined;

  /** Loads running timers and reports the ones that ended while the app was closed. */
  start = async () => {
    this.#running = true;
    await this.#reportMissed();
    this.#setList(await timersAPI.list());
  };

  /** Stops the clock; `start()` picks up again. */
  stop = () => {
    this.#running = false;
    this.#stopTicking();
  };

  tick = () => {
    this.now = Date.now();
    const ended = this.list.filter((t) => t.endsAt <= this.now);
    for (const t of ended) this.#ring(t);
  };

  add = async (title: string, ms: number) => {
    // Ask now, while the user is here, rather than when the timer rings.
    allowNotifications().catch(() => {});
    try {
      const t = await timersAPI.start({ title, ms });
      this.#setList([...this.list, t].sort(bySoonest));
      return t;
    } catch (e) {
      toast.show(messageOf(e));
    }
  };

  addMinute = async (id: string) => {
    await timersAPI.addMinute(id);
    this.#setList(await timersAPI.list());
  };

  cancel = async (id: string) => {
    await timersAPI.cancel(id);
    this.#remove(id);
  };

  async #reportMissed() {
    const missed = await timersAPI.finishedWhileClosed();
    if (!missed.length) return;
    const names = missed.map(timerName).join(', ');
    toast.show(`${names} finished while Tend was closed`);
  }

  async #ring(t: Timer) {
    this.#remove(t.id);
    await timersAPI.ring(t.id);
    if (settings.get(settingKeys.timersSound, true)) chime();
    const name = timerName(t);
    await notify({ title: name, body: 'Time’s up' }).catch(() => toast.show(`${name}: time’s up`));
  }

  #remove(id: string) {
    this.#setList(this.list.filter((t) => t.id !== id));
  }

  #setList(list: Timer[]) {
    this.list = list;
    const shouldTick = this.#running && list.length > 0;
    if (shouldTick) this.#startTicking();
    else this.#stopTicking();
  }

  #startTicking() {
    if (this.#interval) return;
    this.now = Date.now();
    this.#interval = setInterval(this.tick, TICK_MS);
    // After sleep the next interval can be late; waking the window checks at once.
    document.addEventListener('visibilitychange', this.tick);
    window.addEventListener('focus', this.tick);
  }

  #stopTicking() {
    if (!this.#interval) return;
    clearInterval(this.#interval);
    this.#interval = undefined;
    document.removeEventListener('visibilitychange', this.tick);
    window.removeEventListener('focus', this.tick);
  }
}

export const timers = new TimersStore();
