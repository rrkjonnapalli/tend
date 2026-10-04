<script lang="ts">
  import type { IconName } from '#lib/types/icon.type.ts';
  import { icons } from '#lib/icons.ts';

  let {
    name,
    size = 16,
    stroke = 2,
  }: { name: IconName; size?: number; stroke?: number } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true">{@html icons[name]}</svg>
