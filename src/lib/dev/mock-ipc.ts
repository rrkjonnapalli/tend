// Dev only: lets the UI run in a plain browser (vite dev without Tauri) for layout checks.
// Commands answer from in-memory fixtures. Add `?vault=0` to the URL to see the start page.
import { mockIPC } from '@tauri-apps/api/mocks';

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

const handlers: Record<string, Handler> = {
  vault_restore: () => (withVault ? '/Users/demo/Tend' : null),
  vault_current: () => (withVault ? '/Users/demo/Tend' : null),
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
