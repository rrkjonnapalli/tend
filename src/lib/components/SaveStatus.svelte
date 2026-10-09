<script lang="ts">
  // A note's save state in its page bar: quiet "Saved" / "Saving…", or "Not saved" with Retry.
  import type { SaveState } from '#lib/types/save.type.ts';

  let { state, onretry }: { state: SaveState; onretry: () => void } = $props();

  const LABELS: Record<SaveState, string> = { saved: 'Saved', saving: 'Saving…', failed: 'Not saved' };
</script>

<span class="st" class:bad={state === 'failed'} aria-live="polite">
  {LABELS[state]}
  {#if state === 'failed'}<button class="link-btn retry" onclick={onretry}>Retry</button>{/if}
</span>

<style>
  .st {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    margin-right: var(--space-2);
    font: var(--text-meta);
    color: var(--c-muted-2);
    white-space: nowrap;
  }

  .st.bad {
    color: var(--danger);
  }

  .retry {
    font: var(--text-meta);
  }
</style>
