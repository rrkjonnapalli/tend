// Loads that can overlap (a save's reload and a late title fetch's): only the newest one asked
// for is applied, so a slow older reply can't put stale rows back.
export function latestOnly<T>(load: () => Promise<T>, apply: (value: T) => void): () => Promise<void> {
  let asked = 0;
  return async () => {
    asked += 1;
    const mine = asked;
    const value = await load();
    if (mine === asked) apply(value);
  };
}
