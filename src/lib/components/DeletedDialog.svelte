<script lang="ts">
  // Asks whether to keep an open note or task that was deleted elsewhere (see useDeleted).
  import ConfirmDialog from './ConfirmDialog.svelte';

  let {
    kind,
    onkeep,
    ondiscard,
  }: {
    kind: 'note' | 'task';
    onkeep: () => Promise<void>;
    ondiscard: () => void;
  } = $props();

  const COPY = {
    note: {
      title: 'This note was deleted',
      body: 'It was deleted outside Tend or on another device. Keep it as a new note, with the text you have open?',
    },
    task: {
      title: 'This task was deleted',
      body: 'It was deleted on another device. Keep it as a new task, with its fields, description, tags and labels? Its subtasks, comments and attachments are gone.',
    },
  } as const;
</script>

<ConfirmDialog {...COPY[kind]} confirm="Keep" cancel="Discard" onconfirm={onkeep} onclose={ondiscard} />
