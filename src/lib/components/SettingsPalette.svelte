<script lang="ts">
  // Settings → Command palette: every command by group, with its example and shortcut.
  import type { Command, CommandGroup } from '#lib/types/command.type.ts';
  import { commands } from '#lib/commands.ts';
  import { keyParts } from '#lib/util/keys.ts';

  /** Commands in registry order, grouped under the first appearance of each group. */
  function byGroup(list: Command[]): { name: CommandGroup; items: Command[] }[] {
    const groups = new Map<CommandGroup, Command[]>();
    for (const c of list) {
      const items = groups.get(c.group) ?? [];
      items.push(c);
      groups.set(c.group, items);
    }
    return [...groups].map(([name, items]) => ({ name, items }));
  }

  const groups = byGroup(commands);
</script>

<section class="palette">
  {#each groups as g (g.name)}
    <div class="group">
      <h2 class="h2 gh">{g.name}</h2>
      {#each g.items as c (c.id)}
        <div class="crow">
          <span>{c.label}</span>
          <span class="cex">{c.example ?? ''}</span>
          <span class="ckeys">{#if c.keys}{#each keyParts(c.keys) as k, i (i)}<kbd>{k}</kbd>{/each}{/if}</span>
        </div>
      {/each}
    </div>
  {/each}
</section>

<style>
  .palette {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .h2 {
    font: var(--text-section);
    margin: 0;
  }

  .gh {
    margin-bottom: 6px;
  }

  .crow {
    display: grid;
    grid-template-columns: minmax(0, 220px) minmax(0, 1fr) auto;
    align-items: center;
    gap: 16px;
    min-height: 34px;
    padding: 0 16px;
    margin: 0 -16px;
    border-top: 1px solid var(--c-chip);
    font: var(--text-ui);
  }

  .crow:hover {
    background: var(--c-hover);
  }

  .cex {
    font: var(--text-mono-meta);
    color: var(--c-muted-2);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .ckeys {
    display: flex;
    gap: 4px;
  }
</style>
