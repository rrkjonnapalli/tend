// Dev only: lets the UI run in a plain browser (vite dev without Tauri) for layout checks.
// Commands answer from in-memory fixtures. Add `?vault=0` to the URL to see the start page, `?vault=denied` for the error screen.
import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import { addDays, today } from '#lib/util/dates.ts';

type Handler = (args: Record<string, unknown>) => unknown;

const DAY_MS = 86_400_000;
const VAULT_PATH = '/Users/demo/Tend';
const DENIED_VAULT_PATH = '/Users/demo/Documents/Tend';

const vaultParam = new URLSearchParams(location.search).get('vault');
const withVault = vaultParam !== '0';
const settings: Record<string, unknown> = {};
const now = Date.now();
const notes = [
  { id: 'wc', folder: 'work/tauri', name: 'window_config', modifiedAt: now - 120_000 },
  { id: 'pl', folder: 'work/tauri', name: 'plugins', modifiedAt: now - DAY_MS },
  { id: 'rc', folder: 'work', name: 'release_checklist', modifiedAt: now - 1.2 * DAY_MS },
  { id: 'gr', folder: 'home', name: 'groceries', modifiedAt: now - 3 * DAY_MS },
  { id: 'sc', folder: '', name: 'scratch', modifiedAt: now - 14 * DAY_MS },
];
const texts: Record<string, string> = {
  wc: '# Window config\n\nHide native decorations. One **hairline** is the only chrome.\n\n## Checklist\n\n- [x] decorations: false\n- [ ] drag region on the explorer toolbar\n- [ ] global shortcut for capture\n\n> Closing hides the window, so timers keep running.\n\n```json\n"decorations": false,\n"minWidth": 900\n```',
};
const noteTags: Record<string, string[]> = { wc: ['tauri', 'ui'] };

/** Today plus `n` days, as YYYY-MM-DD. */
const day = (n: number) => addDays(today(), n);

const statuses = [
  { value: 'NOT_STARTED', label: 'Not started', color: '#7d8995', position: 0, isDefault: true },
  { value: 'IN_PROGRESS', label: 'In progress', color: '#1259d3', position: 1, isDefault: true },
  { value: 'WAITING', label: 'Waiting', color: '#d57149', position: 2, isDefault: false },
  { value: 'IN_REVIEW', label: 'In review', color: '#6f5ee4', position: 3, isDefault: false },
  { value: 'DONE', label: 'Done', color: '#1e7a55', position: 1000, isDefault: true },
];

type TaskFixture = { id: string; path: string; title: string; orderKey: string } & Record<string, unknown>;

const TASK_DEFAULTS = {
  parentId: null, status: 'NOT_STARTED', dueDate: null, dueTime: null, assigneeId: null, createdAt: now, completedAt: null,
};

const task = (fields: TaskFixture) => ({ ...TASK_DEFAULTS, ...fields });

const tasks = [
  task({ id: 'ship', path: 'work/tend_v0_1', title: 'Ship first build', orderKey: 'a', status: 'IN_PROGRESS', dueDate: day(7), assigneeId: 'priya' }),
  task({ id: 'setup', path: 'work/tend_v0_1', title: 'Set up Tauri + Svelte', orderKey: 'aa', parentId: 'ship', status: 'DONE' }),
  task({ id: 'nx', path: 'work/tend_v0_1', title: 'Notes explorer', orderKey: 'ab', parentId: 'ship', status: 'IN_PROGRESS' }),
  task({ id: 'rem', path: 'work/tend_v0_1', title: 'Reminders', orderKey: 'ac', parentId: 'ship', dueDate: day(0) }),
  task({ id: 'readme', path: 'work/tend_v0_1', title: 'Write README', orderKey: 'b' }),
  task({ id: 'ci', path: 'work/infra', title: 'Set up CI builds', orderKey: 'c', status: 'IN_REVIEW', dueDate: day(9) }),
  task({ id: 'certs', path: 'work/infra', title: 'Get code signing certificates', orderKey: 'd', dueDate: day(-2) }),
  task({ id: 'weekly', path: 'work', title: 'Weekly review', orderKey: 'e', dueDate: day(-1) }),
  task({ id: 'settings', path: 'work/tend_v0_1', title: 'Settings screen', orderKey: 'f', dueDate: day(6) }),
  task({ id: 'board', path: 'work/tend_v0_1', title: 'Board view', orderKey: 'g', dueDate: day(0), status: 'WAITING' }),
  task({ id: 'icon', path: 'work/tend_v0_1', title: 'App icon', orderKey: 'h' }),
  task({ id: 'plan', path: 'work', title: 'Plan v0.2 scope', orderKey: 'i', dueDate: day(4) }),
  task({ id: 'notes', path: 'work', title: 'Write release notes', orderKey: 'j', status: 'DONE' }),
  task({ id: 'milk', path: 'home', title: 'Buy milk', orderKey: 'k' }),
];
let labels = [
  { id: 'lb-review', name: 'review in progress', color: '#d57149', position: 0 },
  { id: 'lb-script', name: 'script', color: '#1259d3', position: 1 },
  { id: 'lb-approval', name: 'pending approval', color: '#da0037', position: 2 },
];
/** Task id → label ids. */
const taskLabels = new Map<string, string[]>([['ship', ['lb-review', 'lb-script', 'lb-approval']], ['ci', ['lb-script']]]);
const labelOrder = (ids: string[]) => labels.filter((l) => ids.includes(l.id)).map((l) => l.id);
const labelLinks = () => [...taskLabels].flatMap(([taskId, ids]) => ids.map((id) => [taskId, id]));

/** Manual My day choices: task id → included. */
const myDay = new Map<string, boolean>();
let createdCount = 0;

const timerList = [
  { id: 'tea', title: 'Tea', endsAt: now + 271_000 },
  { id: 'motor', title: 'Turn off the motor', endsAt: now + 545_000 },
];
const comments: { id: string; task: string; body: string; createdAt: number; editedAt: number | null }[] = [];
const taskFolders = ['home', 'inbox', 'work', 'work/infra', 'work/tend_v0_1'];

const handlers: Record<string, Handler> = {
  vault_restore: () => {
    if (vaultParam === 'denied') return { path: DENIED_VAULT_PATH, error: 'Tend doesn’t have permission to open this folder.' };
    return { path: withVault ? VAULT_PATH : null, error: null };
  },
  vault_open: ({ path }) => ({
    path,
    ok: true,
    confirm: null,
    steps: [
      { item: 'notes', status: 'found', detail: '42 notes' },
      { item: '.tend/attachments/', status: 'found', detail: 'found · 9 files' },
      { item: '.tend/tend.db', status: 'found', detail: 'schema matches' },
    ],
  }),
  'plugin:dialog|open': () => '/Users/demo/Dropbox/Tend',
  'plugin:event|listen': () => 1,
  'plugin:event|unlisten': () => null,
  'plugin:window|is_maximized': () => false,
  'plugin:window|is_fullscreen': () => false,
  'plugin:window|close': () => null,
  'plugin:window|minimize': () => null,
  'plugin:window|toggle_maximize': () => null,
  tags_all: () => ['ci', 'docs', 'home', 'reading', 'release', 'tauri', 'ui', 'v0.1'],
  notes_tree: () => ({ folders: ['home', 'work', 'work/tauri'], notes }),
  item_rename: ({ item, name }) => {
    const target = item as { kind: string; id?: string; path?: string };
    const note = notes.find((n) => n.id === target.id);
    if (note) note.name = name as string;
    return target.id ?? target.path;
  },
  notes_rescan: () => ({ folders: ['home', 'work', 'work/tauri'], notes }),
  note_read: ({ id }) => ({ ...notes.find((n) => n.id === id), text: texts[id as string] ?? '', tags: noteTags[id as string] ?? [] }),
  note_write: ({ id, text }) => {
    texts[id as string] = text as string;
    return notes.find((n) => n.id === id);
  },
  note_tags_set: ({ id, tags }) => (noteTags[id as string] = tags as string[]),
  statuses_list: () => statuses,
  timers_list: () => timerList,
  timers_finished_while_closed: () => [],
  timer_start: ({ title, ms }) => {
    const t = { id: `t${Date.now()}`, title: title as string, endsAt: Date.now() + (ms as number) };
    timerList.push(t);
    return t;
  },
  timer_ring: () => null,
  timer_cancel: ({ id }) => {
    const index = timerList.findIndex((t) => t.id === id);
    if (index >= 0) timerList.splice(index, 1);
  },
  timer_add_minute: () => null,
  skipped_items: () => [
    { path: 'Café notes.md', fix: 'cafe_notes.md' },
    { path: 'Old Stuff/', fix: 'old_stuff/' },
  ],
  notes_fix_names: () => [],
  feedback_send: () => null,
  tasks_data: () => ({ folders: taskFolders, tasks, myDay: [...myDay], labels: labelLinks() }),
  labels_list: () => labels,
  label_add: ({ name, color }) => {
    const label = { id: `lb-${labels.length + 1}`, name: name as string, color: color as string, position: labels.length };
    labels.push(label);
    return label;
  },
  label_rename: ({ id, name }) => Object.assign(labels.find((l) => l.id === id)!, { name }),
  label_recolor: ({ id, color }) => Object.assign(labels.find((l) => l.id === id)!, { color }),
  label_reorder: ({ ids }) => {
    labels = (ids as string[]).map((id, position) => ({ ...labels.find((l) => l.id === id)!, position }));
    return null;
  },
  label_usage: ({ id }) => [...taskLabels.values()].filter((ids) => ids.includes(id as string)).length,
  label_delete: ({ id }) => {
    labels = labels.filter((l) => l.id !== id);
    for (const [taskId, ids] of taskLabels) taskLabels.set(taskId, ids.filter((x) => x !== id));
    return null;
  },
  task_labels_set: ({ id, labels: ids }) => {
    const ordered = labelOrder(ids as string[]);
    taskLabels.set(id as string, ordered);
    return ordered;
  },
  task_create: ({ task: input }) => {
    createdCount += 1;
    const fields = input as { path: string; title: string; parentId?: string };
    const created = task({ id: `new${createdCount}`, orderKey: '0', ...fields, parentId: fields.parentId ?? null });
    tasks.unshift(created);
    return created;
  },
  task_update: ({ id, patch }) => {
    const t = tasks.find((x) => x.id === id)!;
    Object.assign(t, patch);
    return t;
  },
  my_day_set: ({ id, included }) => {
    if (included === null) myDay.delete(id as string);
    else myDay.set(id as string, included as boolean);
    return null;
  },
  task_get: ({ id }) => ({ ...tasks.find((t) => t.id === id), description: id === 'ship' ? 'First signed build for macOS and Linux.' : '', tags: id === 'ship' ? ['v0.1'] : [], labels: taskLabels.get(id as string) ?? [] }),
  people_list: () => [
    { id: 'priya', firstName: 'Priya', lastName: 'Nair', nickname: null, dobYear: 1995, dobMonth: 10, dobDay: 10, phone: '+91 98450 12345', email: 'priya@example.com' },
    { id: 'ravi', firstName: 'Ravi', lastName: 'Kumar', nickname: null, dobYear: null, dobMonth: 3, dobDay: 2, phone: null, email: 'ravi@example.com' },
  ],
  person_get: ({ id }) => ({
    id, firstName: 'Ravi', lastName: 'Kumar', nickname: null, dobYear: 1995, dobMonth: 10, dobDay: 9, phone: '+91 98765 43210', email: 'ravi.kumar@example.com',
    tob: '06:30', notes: 'Prefers calls after 6.', contacts: [
      { id: 'c1', kind: 'phone', value: '+91 98765 43210', label: 'personal' },
      { id: 'c2', kind: 'phone', value: '+91 80 4123 9876', label: 'work' },
      { id: 'c3', kind: 'email', value: 'ravi.kumar@example.com', label: 'personal' },
      { id: 'c4', kind: 'address', value: '12 Lake View Rd, Bengaluru 560034', label: 'home' },
    ],
  }),
  comments_list: ({ task }) => comments.filter((c) => c.task === task).reverse(),
  comment_add: ({ task, body }) => {
    comments.push({ id: `c${comments.length}`, task: task as string, body: body as string, createdAt: Date.now(), editedAt: null });
  },
  comment_edit: ({ id, body }) => {
    const c = comments.find((x) => x.id === id)!;
    Object.assign(c, { body, editedAt: Date.now() });
  },
  attachments_list: () => [],
  settings_all: () => settings,
  settings_set: ({ key, value }) => {
    settings[key as string] = value;
  },
};

mockIPC((cmd, args) => {
  const handler = handlers[cmd];
  if (!handler) throw { key: 'MOCK', message: `no mock for ${cmd}` };
  return handler((args ?? {}) as Record<string, unknown>);
});
mockWindows('main');
