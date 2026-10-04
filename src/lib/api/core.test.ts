import { afterEach, expect, it, vi } from 'vitest';

const invoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({ invoke }));

const { call } = await import('./core.ts');
const { errors } = await import('#lib/errors/registry.ts');

afterEach(() => {
  vi.useRealTimers();
  invoke.mockReset();
});

it('returns the reply and passes the args through', async () => {
  invoke.mockResolvedValue('ok');
  await expect(call('note_read', { args: { id: 'a' } })).resolves.toBe('ok');
  expect(invoke).toHaveBeenCalledWith('note_read', { id: 'a' });
});

it('fails with a timeout error when Rust never answers', async () => {
  vi.useFakeTimers();
  invoke.mockReturnValue(new Promise(() => {}));
  const reply = call('notes_tree', { timeoutMs: 1000 });
  const check = expect(reply).rejects.toMatchObject({ key: errors.TIMEOUT.key });
  await vi.advanceTimersByTimeAsync(1000);
  await check;
});

it('turns a Rust error into an AppError with its key', async () => {
  invoke.mockRejectedValue({ key: 'NAME_TAKEN', message: 'Something with this name already exists here' });
  await expect(call('note_create')).rejects.toMatchObject({ key: 'NAME_TAKEN', name: 'AppError' });
});

it('waits as long as it takes when the timeout is null', async () => {
  vi.useFakeTimers();
  let answer: (v: string) => void = () => {};
  invoke.mockReturnValue(new Promise((resolve) => {
    answer = resolve;
  }));
  const reply = call('vault_restore', { timeoutMs: null });
  await vi.advanceTimersByTimeAsync(10 * 60_000);
  answer('opened');
  await expect(reply).resolves.toBe('opened');
});
