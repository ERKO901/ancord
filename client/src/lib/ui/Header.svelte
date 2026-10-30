<script lang="ts">
  import type { Snippet } from "svelte";
  import Icon from "./Icon.svelte";

  let { title, onback, hideBackOnWide = false, right }: {
    title: string;
    onback?: () => void;
    hideBackOnWide?: boolean;
    right?: Snippet;
  } = $props();
</script>

<header class="hdr">
  <div class="side">
    {#if onback}
      <button class="icon-btn" class:wide-hide={hideBackOnWide} onclick={onback} aria-label="Zpět"><Icon name="back" /></button>
    {/if}
  </div>
  <h1>{title}</h1>
  <div class="side end">{@render right?.()}</div>
</header>

<style>
  .hdr {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    min-height: calc(56px + var(--safe-top));
    padding: var(--safe-top) var(--space-2) 0;
    flex: none;
  }
  h1 {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: 600;
    max-width: 60vw;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .side { display: flex; align-items: center; }
  .end { justify-content: flex-end; }
  @media (min-width: 900px) {
    .wide-hide { display: none; }
  }
</style>
