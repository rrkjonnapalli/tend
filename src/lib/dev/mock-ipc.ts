// Dev only: lets the UI run in a plain browser (vite dev without Tauri) for layout checks.
// Commands answer from in-memory fixtures. Add `?vault=0` to the URL to see the start page.
import { mockIPC } from '@tauri-apps/api/mocks';

type Handler = (args: Record<string, unknown>) => unknown;

const withVault = new URLSearchParams(location.search).get('vault') !== '0';
const settings: Record<string, unknown> = {};

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
