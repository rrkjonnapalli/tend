// Dev only: lets the UI run in a plain browser (vite dev without Tauri) for layout checks.
// Commands answer from in-memory fixtures. Add `?vault=0` to the URL to see the start page.
import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';

type Handler = (args: Record<string, unknown>) => unknown;

const withVault = new URLSearchParams(location.search).get('vault') !== '0';
const settings: Record<string, unknown> = {};
const now = Date.now();
const notes = [
  { id: 'wc', folder: 'work/tauri', name: 'window_config', title: 'Window config', modifiedAt: now - 120_000 },
  { id: 'pl', folder: 'work/tauri', name: 'plugins', title: 'Plugins', modifiedAt: now - 86_400_000 },
  { id: 'rc', folder: 'work', name: 'release_checklist', title: 'Release checklist', modifiedAt: now - 1.2 * 86_400_000 },
  { id: 'gr', folder: 'home', name: 'groceries', title: 'Groceries', modifiedAt: now - 3 * 86_400_000 },
  { id: 'sc', folder: '', name: 'scratch', title: 'Scratch', modifiedAt: now - 14 * 86_400_000 },
];
const texts: Record<string, string> = {
  wc: '# Window config\n\nHide native decorations. One **hairline** is the only chrome.\n\n## Checklist\n\n- [x] decorations: false\n- [ ] drag region on the explorer toolbar\n- [ ] global shortcut for capture\n\n> Closing hides the window, so timers keep running.\n\n```json\n"decorations": false,\n"minWidth": 900\n```',
};
const noteTags: Record<string, string[]> = { wc: ['tauri', 'ui'] };

const day = (n: number) => {
  const d = new Date();
  d.setDate(d.getDate() + n);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
};
const statuses = [
  { value: 'NOT_STARTED', label: 'Not started', color: '#7d8995', position: 0, isDefault: true },
  { value: 'IN_PROGRESS', label: 'In progress', color: '#1259d3', position: 1, isDefault: true },
  { value: 'WAITING', label: 'Waiting', color: '#d57149', position: 2, isDefault: false },
  { value: 'IN_REVIEW', label: 'In review', color: '#6f5ee4', position: 3, isDefault: false },
  { value: 'DONE', label: 'Done', color: '#1e7a55', position: 1000, isDefault: true },
];
const task = (id: string, path: string, title: string, key: string, o: Record<string, unknown> = {}) => ({
  id, path, title, orderKey: key, parentId: null, status: 'NOT_STARTED', dueDate: null, dueTime: null, assigneeId: null, createdAt: now, completedAt: null, ...o,
});
const tasks = [
  task('ship', 'work/tend_v0_1', 'Ship first build', 'a', { status: 'IN_PROGRESS', dueDate: day(7), assigneeId: 'priya' }),
  task('setup', 'work/tend_v0_1', 'Set up Tauri + Svelte', 'aa', { parentId: 'ship', status: 'DONE' }),
  task('nx', 'work/tend_v0_1', 'Notes explorer', 'ab', { parentId: 'ship', status: 'IN_PROGRESS' }),
  task('rem', 'work/tend_v0_1', 'Reminders', 'ac', { parentId: 'ship' }),
  task('readme', 'work/tend_v0_1', 'Write README', 'b'),
  task('ci', 'work/infra', 'Set up CI builds', 'c', { status: 'IN_REVIEW', dueDate: day(9) }),
  task('certs', 'work/infra', 'Get code signing certificates', 'd', { dueDate: day(-2) }),
  task('weekly', 'work', 'Weekly review', 'e', { dueDate: day(-1) }),
  task('settings', 'work/tend_v0_1', 'Settings screen', 'f', { dueDate: day(6) }),
  task('board', 'work/tend_v0_1', 'Board view', 'g', { dueDate: day(0), status: 'WAITING' }),
  task('icon', 'work/tend_v0_1', 'App icon', 'h'),
  task('plan', 'work', 'Plan v0.2 scope', 'i', { dueDate: day(4) }),
  task('notes', 'work', 'Write release notes', 'j', { status: 'DONE' }),
  task('milk', 'home', 'Buy milk', 'k'),
];
const timerList = [
  { id: 'tea', title: 'Tea', endsAt: now + 271_000 },
  { id: 'motor', title: 'Turn off the motor', endsAt: now + 545_000 },
];
const comments: { id: string; task: string; body: string; createdAt: number; editedAt: number | null }[] = [];
const taskFolders = ['home', 'inbox', 'work', 'work/infra', 'work/tend_v0_1'];

const handlers: Record<string, Handler> = {
  vault_restore: () => (withVault ? '/Users/demo/Tend' : null),
  vault_open: ({ path }) => ({
    path,
    ok: true,
    steps: [
      { item: 'notes/', status: 'found', detail: 'found · 42 notes' },
      { item: '.attachments/', status: 'found', detail: 'found · 9 files' },
      { item: 'db/app.sqlite', status: 'found', detail: 'schema matches' },
    ],
  }),
  'plugin:dialog|open': () => '/Users/demo/Dropbox/Tend',
  'plugin:event|listen': () => 1,
  'plugin:event|unlisten': () => null,
  tags_all: () => ['ci', 'docs', 'home', 'reading', 'release', 'tauri', 'ui', 'v0.1'],
  notes_tree: () => ({ folders: ['home', 'work', 'work/tauri'], notes }),
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
  timer_cancel: ({ id }) => timerList.splice(timerList.findIndex((t) => t.id === id), 1),
  timer_add_minute: () => null,
  skipped_items: () => ['Bad Name.md', 'My Folder/'],
  feedback_send: () => null,
  tasks_data: () => ({ folders: taskFolders, tasks, myDay: [] }),
  task_update: ({ id, patch }) => {
    const t = tasks.find((x) => x.id === id)!;
    Object.assign(t, patch);
    return t;
  },
  my_day_set: () => null,
  task_get: ({ id }) => ({ ...tasks.find((t) => t.id === id), description: id === 'ship' ? 'First signed build for macOS and Linux.' : '', tags: id === 'ship' ? ['v0.1'] : [] }),
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

export function register(more: Record<string, Handler>) {
  Object.assign(handlers, more);
}

mockIPC((cmd, args) => {
  const handler = handlers[cmd];
  if (!handler) throw { code: 'mock', message: `no mock for ${cmd}` };
  return handler((args ?? {}) as Record<string, unknown>);
});
mockWindows('main');
