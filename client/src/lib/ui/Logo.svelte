<script lang="ts">
  // Značka ancord: písmeno „a“ (kruh a dřík).
  let { size = 40, tile = true }: { size?: number; tile?: boolean } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 64 64" fill="none" aria-hidden="true">
  {#if tile}<rect width="64" height="64" rx="16" fill="var(--accent)" />{/if}
  <g stroke={tile ? "var(--on-accent)" : "var(--accent)"} stroke-width="5" stroke-linecap="round">
    <circle cx="29" cy="34" r="9.500" />
    <path d="M43 20v24" />
  </g>
</svg>
