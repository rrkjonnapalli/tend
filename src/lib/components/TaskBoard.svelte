<script lang="ts">
  // The board: one column per status. Cards move with the pointer; a dashed placeholder shows
  // where they land. Dropping sets the status from the column and the order from the position.
  import { goto } from '$app/navigation';
  import type { Task } from '#lib/types/task.type.ts';
  import { messageOf } from '#lib/api/core.ts';
  import StatusDot from './StatusDot.svelte';
  import Avatar from './Avatar.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { people } from '#lib/stores/people.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { dueLabel } from '#lib/util/dates.ts';
  import { isDone, isOverdue } from '#lib/util/task-views.ts';
  import { keyBetween } from '#lib/util/order-key.ts';
  import { relativeTo } from '#lib/util/names.ts';
  import { routes } from '#lib/util/routes.ts';

  let { items, folder }: { items: Task[]; folder: string } = $props();

  people.load();

  /** Where the dragged card would land. */
  let drag = $state<{ id: string; col: string; idx: number } | null>(null);
  let adding = $state<string | null>(null);
  let draft = $state('');

  const byKey = (a: Task, b: Task) => (a.orderKey < b.orderKey ? -1 : a.orderKey > b.orderKey ? 1 : 0);

  function cardsOf(status: string): Task[] {
    const list = items.filter((t) => t.status === status && t.id !== drag?.id).sort(byKey);
    if (drag && drag.col === status) {
      const moving = items.find((t) => t.id === drag!.id);
      if (moving) list.splice(Math.min(drag.idx, list.length), 0, moving);
    }
    return list;
  }

  function target(ev: PointerEvent, id: string) {
    const cols = [...document.querySelectorAll<HTMLElement>('[data-col]')];
    if (!cols.length) return null;
    let best = cols[0];
    let bestD = Infinity;
    for (const el of cols) {
      const r = el.getBoundingClientRect();
      const d = ev.clientX < r.left ? r.left - ev.clientX : ev.clientX > r.right ? ev.clientX - r.right : 0;
      if (d < bestD) [best, bestD] = [el, d];
    }
    // Measure as if the placeholder weren't there, so the target doesn't flip back and forth.
    const ph = best.querySelector<HTMLElement>(`[data-card="${id}"]`);
    const phR = ph?.getBoundingClientRect();
    const shift = phR ? phR.height + 6 : 0;
    const mids = [...best.querySelectorAll<HTMLElement>('[data-card]')]
      .filter((c) => c !== ph)
      .map((c) => {
        const r = c.getBoundingClientRect();
        const m = r.top + r.height / 2;
        return phR && r.top > phR.top ? m - shift : m;
      });
    return { id, col: best.dataset.col!, idx: mids.filter((m) => m < ev.clientY).length };
  }

  function grab(e: PointerEvent, task: Task) {
    if (e.button !== 0) return;
    const card = e.currentTarget as HTMLElement;
    const [x0, y0] = [e.clientX, e.clientY];
    let ghost: HTMLElement | null = null;
    let off = { x: 0, y: 0 };
    let frame = 0;
    let last: PointerEvent | null = null;

    const step = () => {
      frame = 0;
      if (!last) return;
      const t = target(last, task.id);
      if (t && (!drag || drag.col !== t.col || drag.idx !== t.idx)) drag = t;
    };
    const move = (ev: PointerEvent) => {
      if (!ghost) {
        if (Math.abs(ev.clientX - x0) + Math.abs(ev.clientY - y0) < 5) return;
        const r = card.getBoundingClientRect();
        off = { x: x0 - r.left, y: y0 - r.top };
        ghost = card.cloneNode(true) as HTMLElement;
        ghost.classList.add('ghost');
        ghost.removeAttribute('data-card');
        Object.assign(ghost.style, { position: 'fixed', left: '0', top: '0', margin: '0', zIndex: '1000', pointerEvents: 'none', width: `${r.width}px` });
        document.body.appendChild(ghost);
        document.body.classList.add('reordering');
        drag = { id: task.id, col: task.status, idx: cardsOf(task.status).findIndex((t) => t.id === task.id) };
      }
      ev.preventDefault();
      ghost.style.transform = `translate(${ev.clientX - off.x}px, ${ev.clientY - off.y}px) rotate(1.5deg)`;
      last = ev;
      if (!frame) frame = requestAnimationFrame(step);
    };
    const up = () => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
      window.removeEventListener('pointercancel', up);
      if (frame) cancelAnimationFrame(frame);
      ghost?.remove();
      document.body.classList.remove('reordering');
      if (!ghost) return;
      window.addEventListener('click', (c) => c.stopPropagation(), { capture: true, once: true });
      const d = drag;
      drag = null;
      if (d) drop(task, d.col, d.idx);
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
    window.addEventListener('pointercancel', up);
  }

  async function drop(task: Task, col: string, idx: number) {
    const others = items.filter((t) => t.status === col && t.id !== task.id).sort(byKey);
    const orderKey = keyBetween(others[idx - 1]?.orderKey ?? null, others[idx]?.orderKey ?? null);
    try {
      await tasks.update(task.id, col === task.status ? { orderKey } : { orderKey, status: col });
    } catch (e) {
      toast.show(messageOf(e));
    }
  }

  async function add(status: string) {
    const title = draft.trim();
    if (!title) return;
    const last = cardsOf(status).at(-1)?.orderKey ?? null;
    try {
      await tasks.create({ path: folder, title, status, orderKey: keyBetween(last, null) });
      draft = '';
    } catch (e) {
      toast.show(messageOf(e));
    }
  }

  function focusOnMount(node: HTMLInputElement) {
    node.focus();
  }
</script>

<div class="board">
  {#each tasks.statuses as s (s.value)}
    {@const cards = cardsOf(s.value)}
    <section class="col" data-col={s.value} aria-label={s.label}>
      <header class="colh"><StatusDot value={s.value} /><span>{s.label}</span><span class="vc">{cards.length || ''}</span></header>
      <div class="cards">
        {#each cards as t (t.id)}
          {@const who = people.byId(t.assigneeId)}
          {@const rel = relativeTo(t.path, folder)}
          {@const subs = tasks.subtaskCount(t.id)}
          <div
            class="card"
            class:lifted={drag?.id === t.id}
            class:isdone={isDone(t)}
            data-card={t.id}
            role="button"
            tabindex="0"
            onpointerdown={(e) => grab(e, t)}
            onclick={() => goto(routes.task(t.id))}
            onkeydown={(e) => e.key === 'Enter' && goto(routes.task(t.id))}>
            <span class="ctitle">{t.title}</span>
            {#if t.dueDate || subs || rel || who}
              <span class="cmeta">
                {#if t.dueDate}<span class:warn={isOverdue(t, tasks.day)}>{dueLabel(t.dueDate)}</span>{/if}
                {#if subs}<span>{subs}</span>{/if}
                {#if rel}<span>{rel}</span>{/if}
                {#if who}<span class="cav"><Avatar person={who} size={18} /></span>{/if}
              </span>
            {/if}
          </div>
        {/each}
        {#if cards.length === 0 && adding !== s.value}<span class="cempty">No tasks</span>{/if}
        {#if adding === s.value}
          <input
            class="cadd"
            aria-label="New task in {s.label}"
            placeholder="Task title, ↵ to add"
            bind:value={draft}
            use:focusOnMount
            onkeydown={(e) => {
              if (e.key === 'Enter') add(s.value);
              if (e.key === 'Escape') ((adding = null), (draft = ''));
            }}
            onblur={() => !draft.trim() && (adding = null)} />
        {:else}
          <button class="caddb" onclick={() => ((adding = s.value), (draft = ''))}>+ add task</button>
        {/if}
      </div>
    </section>
  {/each}
</div>

<style>
  .board {
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
    align-items: stretch;
    gap: 12px;
    overflow-x: auto;
    padding-bottom: 8px;
    user-select: none;
  }

  .col {
    flex: 0 0 248px;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 8px 8px;
    background: var(--c-hover);
    border-radius: var(--radius-md);
  }

  .colh {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 4px;
    font: 500 13px/18px var(--font-heading);
  }

  .vc {
    font: 400 12px/16px var(--font-ui);
    color: var(--c-muted-2);
  }

  .cards {
    flex: 1 1 auto;
    min-height: 44px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px 10px;
    background: var(--c-page);
    border: 1px solid var(--border);
    border-radius: 6px;
    font: 400 14px/20px var(--font-ui);
    cursor: pointer;
    touch-action: none;
  }

  .card:hover {
    border-color: var(--c-faint);
  }

  .card.lifted {
    background: transparent;
    border: 1px dashed var(--accent);
  }

  .card.lifted > * {
    visibility: hidden;
  }

  :global(.card.ghost) {
    border-color: var(--accent) !important;
    box-shadow: inset 2px 0 0 var(--accent);
    opacity: 0.96;
    will-change: transform;
  }

  .isdone .ctitle {
    color: var(--c-muted-2);
  }

  .cmeta {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    font: 400 12px/16px var(--font-ui);
    color: var(--c-muted-2);
  }

  .warn {
    color: var(--c-overdue);
  }

  .cav {
    display: grid;
    margin-left: auto;
  }

  .cadd {
    width: 100%;
    height: 36px;
    padding: 0 10px;
    border: 1px solid var(--primary);
    border-radius: 6px;
    outline: none;
    background: var(--c-page);
    font: 400 14px/20px var(--font-ui);
  }

  .cadd::placeholder {
    color: var(--c-faint);
  }

  .caddb {
    height: 30px;
    padding: 0 6px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    text-align: left;
    color: var(--c-muted-2);
    font: 400 13px/18px var(--font-ui);
    cursor: pointer;
  }

  .caddb:hover {
    color: var(--text);
    background: var(--c-selected);
  }

  .cempty {
    padding: 12px 4px;
    font: 400 12px/16px var(--font-ui);
    color: var(--c-faint);
    text-align: center;
  }
</style>
