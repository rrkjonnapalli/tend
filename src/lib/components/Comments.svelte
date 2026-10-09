<script lang="ts">
  // Comment composer and the comments, newest first. Each comment edits inline.
  import MarkdownEditor from './MarkdownEditor.svelte';
  import MarkdownField from './MarkdownField.svelte';
  import EditControls from './EditControls.svelte';
  import { commentsAPI } from '#lib/api/comments.ts';
  import type { Comment } from '#lib/types/comment.type.ts';
  import { ago } from '#lib/util/dates.ts';
  import { label as keyLabel } from '#lib/util/keys.ts';

  let { task, onchange }: { task: string; onchange?: (count: number) => void } = $props();

  let list = $state<Comment[]>([]);
  let draft = $state('');
  let composerKey = $state(0);
  const fields: Record<string, MarkdownField> = $state({});
  const editing: Record<string, boolean> = $state({});

  async function load() {
    list = await commentsAPI.list(task);
    onchange?.(list.length);
  }

  // Reload when another task's comments are shown.
  $effect(() => {
    void task;
    load();
  });

  async function post() {
    if (!draft.trim()) return;
    await commentsAPI.add({ task, body: draft });
    draft = '';
    composerKey += 1;
    await load();
  }

  async function save(c: Comment, body: string) {
    await commentsAPI.edit({ id: c.id, body });
    await load();
  }
</script>

<div class="composer">
  {#key composerKey}
    <MarkdownEditor value={draft} onchange={(t) => (draft = t)} onsave={post} placeholder="Write a comment…" minHeight={72} label="Write a comment" />
  {/key}
  <div class="foot">
    <span class="hint">{keyLabel('mod+enter')}</span>
    <button class="post" disabled={!draft.trim()} onclick={post}>Comment</button>
  </div>
</div>

{#each list as c (c.id)}
  <div class="cm reveal-on-hover">
    <div class="meta">
      <span class="caption muted who">You · {ago(c.createdAt)}{c.editedAt ? ' · edited' : ''}</span>
      <EditControls
        editing={!!editing[c.id]}
        hover
        what="comment"
        onedit={() => fields[c.id]?.edit()}
        oncancel={() => fields[c.id]?.cancel()}
        onsave={() => fields[c.id]?.save()} />
    </div>
    <MarkdownField
      bind:this={fields[c.id]}
      bind:editing={editing[c.id]}
      value={c.body}
      onsave={(body) => save(c, body)}
      minHeight={0}
      label="Edit comment" />
  </div>
{/each}

<style>
  .composer {
    display: flex;
    flex-direction: column;
    margin-bottom: 8px;
  }

  .foot {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    padding-top: 2px;
  }

  .hint {
    font: var(--text-meta);
    color: var(--c-muted-2);
    padding-right: 4px;
  }

  .post {
    height: 26px;
    padding: 0 12px;
    border: 0;
    border-radius: var(--radius-sm);
    background: var(--primary);
    color: var(--on-primary);
    font: 500 13px/18px var(--font-ui);
    cursor: pointer;
  }

  .post:hover {
    background: var(--primary-hover);
  }

  .post:disabled {
    background: var(--border);
    color: var(--c-muted-2);
    cursor: default;
  }

  .cm {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 16px;
    margin: 0 -16px;
  }

  .meta {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 24px;
  }

  .who {
    flex: 1;
  }
</style>
