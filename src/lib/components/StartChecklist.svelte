<script lang="ts">
  // The vault check on the start page: steps up to `shown` show their result, the next one spins.
  import Icon from './Icon.svelte';
  import type { Step, StepState } from '#lib/types/vault.type.ts';

  let { steps, shown }: { steps: Step[]; shown: number } = $props();

  function stepState(step: Step, index: number): StepState {
    if (index < shown) return step.status === 'error' ? 'bad' : 'ok';
    if (index === shown) return 'run';
    return 'wait';
  }

  function stepText(step: Step, state: StepState): string {
    if (state === 'run') return 'checking…';
    if (state === 'wait') return '';
    return step.detail;
  }
</script>

<div class="steps">
  {#each steps as step, i (step.item)}
    {@const state = stepState(step, i)}
    <div class="step {state}">
      <span class="ic">
        {#if state === 'ok'}<span class="okic"><Icon name="check" size={13} stroke={2.5} /></span>{/if}
        {#if state === 'bad'}<span class="badic"><Icon name="close" size={13} stroke={2.5} /></span>{/if}
      </span>
      <span class="mono name">{step.item}</span>
      <span class="st">{stepText(step, state)}</span>
    </div>
  {/each}
</div>

<style>
  .mono {
    font: var(--text-path);
    color: var(--text-muted);
  }

  .steps {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-left: 4px;
  }

  .step {
    display: flex;
    align-items: center;
    gap: 10px;
    font: var(--text-ui);
  }

  .step .name {
    flex: 0 0 120px;
  }

  .step .ic {
    flex: 0 0 14px;
    display: grid;
    place-items: center;
  }

  .okic {
    display: grid;
    color: var(--success);
  }

  .badic {
    display: grid;
    color: var(--danger);
  }

  .step .st {
    color: var(--c-muted-2);
    font-size: 13px;
  }

  .step.wait .name {
    color: var(--c-muted-2);
  }

  .step.wait .ic::before {
    content: '';
    width: 6px;
    height: 6px;
    border-radius: 3px;
    background: var(--border);
  }

  .step.run .ic::before {
    content: '';
    width: 10px;
    height: 10px;
    border: 2px solid var(--border);
    border-top-color: var(--primary);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
