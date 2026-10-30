<script lang="ts">
  import type { Snippet } from "svelte";
  import Icon from "./Icon.svelte";

  let { title, subtitle, onback, hideBackOnWide = false, large = false, lead, right }: {
    title: string;
    subtitle?: string;
    onback?: () => void;
    hideBackOnWide?: boolean;
    /** Nadpis nejvyšší úrovně: vlevo, větší. */
    large?: boolean;
    lead?: Snippet;
    right?: Snippet;
  } = $props();
</script>

<header class="hdr" class:large>
  {#if onback}
    <button class="icon-btn back" class:wide-hide={hideBackOnWide} onclick={onback} aria-label="Zpět"><Icon name="back" /></button>
  {/if}
  {@render lead?.()}
  <div class="titles">
    <h1>{title}</h1>
    {#if subtitle}<p class="sub" role="status">{subtitle}</p>{/if}
  </div>
  <div class="end">{@render right?.()}</div>
</header>

<style>
  .hdr {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: calc(60px + var(--safe-top));
    padding: var(--safe-top) var(--space-2) 0 var(--space-3);
    flex: none;
  }
  .back { margin-left: calc(var(--space-2) * -1); }
  .titles { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  h1 {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sub { margin: 0; font-size: var(--fs-sm); color: var(--muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .large { min-height: calc(72px + var(--safe-top)); padding-left: var(--space-4); gap: var(--space-3); }
  .large h1 { font-size: var(--fs-xl); font-weight: 700; letter-spacing: -0.01em; }
  .end { display: flex; align-items: center; }
  @media (min-width: 900px) {
    .wide-hide { display: none; }
  }
</style>
