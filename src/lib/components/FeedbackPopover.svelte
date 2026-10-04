<script lang="ts">
  // Feedback: Bug / Idea / Other, a text area and Send (⌘↵ also sends). Saved in the vault.
  import Icon from './Icon.svelte';
  import Popover from './Popover.svelte';
  import { feedbackAPI } from '#lib/api/feedback.ts';
  import type { FeedbackKind } from '#lib/types/feedback.type.ts';
  import { messageOf } from '#lib/api/core.ts';
  import { matches } from '#lib/util/keys.ts';

  let { anchor, onclose }: { anchor: HTMLElement | undefined; onclose: () => void } = $props();

  const KINDS: [FeedbackKind, string][] = [
    ['bug', 'Bug'],
    ['idea', 'Idea'],
    ['other', 'Other'],
  ];

  let kind = $state<FeedbackKind>('idea');
  let draft = $state('');
  let sent = $state(false);
  let error = $state('');

  async function send() {
    if (!draft.trim()) return;
    try {
      await feedbackAPI.send({ kind, body: draft });
      draft = '';
      sent = true;
      error = '';
    } catch (e) {
      error = messageOf(e);
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (matches(e, 'mod+enter')) {
      e.preventDefault();
      send();
    }
  }

  function focusOnMount(node: HTMLTextAreaElement) {
    node.focus();
  }
</script>

<Popover {anchor} placement="top-end" offset={8} label="Send feedback" {onclose}>
  <div class="fb">
    <div class="head">
      <span>Feedback</span>
      <button class="icon-btn sm x" aria-label="Close" title="Close" onclick={onclose}><Icon name="close" size={12} stroke={2.25} /></button>
    </div>
    {#if sent}
      <span class="ok">Thanks, saved. You can send another anytime.</span>
      <div class="foot"><button class="kind" onclick={() => (sent = false)}>Write another</button></div>
    {:else}
      <div class="kinds" role="group" aria-label="Kind">
        {#each KINDS as [k, label] (k)}
          <button class="kind" class:on={kind === k} aria-pressed={kind === k} onclick={() => (kind = k)}>{label}</button>
        {/each}
      </div>
      <textarea aria-label="Feedback" placeholder="A bug, an idea, something you’d like…" bind:value={draft} {onkeydown} use:focusOnMount></textarea>
      {#if error}<span class="err">{error}</span>{/if}
      <div class="foot">
        <span class="muted small">Saved in your vault</span>
        <button class="btn-primary send" disabled={!draft.trim()} onclick={send}>Send</button>
      </div>
    {/if}
  </div>
</Popover>

<style>
  .fb {
    width: 320px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    font: 400 13px/18px var(--font-ui);
  }

  .head {
    display: flex;
    align-items: center;
    gap: 8px;
    font: 500 14px/20px var(--font-heading);
  }

  .x {
    margin-left: auto;
  }

  .kinds {
    display: flex;
    gap: 4px;
  }

  .kind {
    height: 24px;
    padding: 0 8px;
    border: 1px solid var(--border);
    border-radius: 12px;
    background: var(--c-page);
    color: var(--text-muted);
    font: 400 12px/16px var(--font-ui);
    cursor: pointer;
  }

  .kind:hover {
    color: var(--text);
    border-color: var(--c-faint);
  }

  .kind.on {
    color: var(--primary);
    border-color: var(--primary);
    background: var(--primary-soft);
  }

  textarea {
    width: 100%;
    min-height: 96px;
    padding: 8px;
    border: 1px solid var(--border);
    border-radius: 6px;
    outline: none;
    resize: vertical;
    font: 400 13px/20px var(--font-ui);
    background: var(--c-page);
  }

  textarea:focus {
    border-color: var(--primary);
  }

  textarea::placeholder {
    color: var(--c-muted-2);
  }

  .foot {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .small {
    font-size: 12px;
  }

  .send {
    margin-left: auto;
  }

  .ok {
    color: var(--success);
  }

  .err {
    color: var(--danger);
    font-size: 12px;
  }
</style>
