<script lang="ts" generics="T">
  import { tick, untrack, type Snippet } from "svelte";
  import { HeightIndex } from "./virtual";

  let {
    items,
    key,
    row,
    estimate = 64,
    overscan = 600,
    onTop,
    label,
  }: {
    items: readonly T[];
    key: (item: T) => string;
    row: Snippet<[T]>;
    estimate?: number;
    overscan?: number;
    /** Uživatel dorolal na začátek: načíst starší. */
    onTop?: () => void;
    label: string;
  } = $props();

  const index = new HeightIndex(untrack(() => estimate));
  let host: HTMLDivElement | undefined = $state();
  let scrollTop = $state(0);
  let viewport = $state(0);
  let version = $state(0); // zvyšuje se při změně výšek
  let firstKey: string | undefined;
  let lastKey: string | undefined;
  let pinned = true;
  let raf = 0;

  const keys = $derived(items.map(key));
  $effect.pre(() => {
    void version;
    index.rebuild(keys);
  });

  const win = $derived.by(() => {
    void version;
    void keys;
    return index.range(scrollTop, viewport, overscan);
  });
  const total = $derived.by(() => { void version; void keys; return index.total; });
  const top = $derived.by(() => { void version; void keys; return index.offsets[win.start] ?? 0; });
  const bottom = $derived.by(() => { void version; void keys; return total - (index.offsets[win.end] ?? total); });

  // Změna obsahu: přilepit ke spodku, nebo zachovat pozici při doplnění starších.
  $effect(() => {
    const ks = keys;
    const el = host;
    if (!el) return;
    untrack(() => {
      const prevFirst = firstKey;
      const prevLast = lastKey;
      firstKey = ks[0];
      lastKey = ks[ks.length - 1];
      if (prevFirst === undefined) {
        void tick().then(() => { if (host) host.scrollTop = host.scrollHeight; });
      } else if (prevFirst !== ks[0]) {
        const k = ks.indexOf(prevFirst);
        if (k > 0) {
          const delta = index.offsets[k] ?? 0;
          void tick().then(() => { if (host) host.scrollTop += delta; });
        }
      } else if (prevLast !== lastKey && pinned) {
        void tick().then(() => host?.scrollTo({ top: host.scrollHeight }));
      }
    });
  });

  function onScroll() {
    if (!host) return;
    scrollTop = host.scrollTop;
    pinned = host.scrollHeight - host.scrollTop - host.clientHeight < 48;
    if (host.scrollTop < 200) onTop?.();
  }

  function schedule() {
    if (raf) return;
    raf = requestAnimationFrame(() => { raf = 0; version++; });
  }

  const ro = new ResizeObserver((entries) => {
    for (const e of entries) {
      const el = e.target as HTMLElement;
      const k = el.dataset["k"];
      if (k === undefined) continue;
      const h = (e.borderBoxSize[0]?.blockSize ?? el.offsetHeight);
      const diff = index.measure(k, h);
      if (diff === 0) continue;
      // Řádek nad horním okrajem změnil výšku: držet obsah na místě.
      const i = keys.indexOf(k);
      if (host && i >= 0 && (index.offsets[i] ?? 0) < host.scrollTop && !pinned) host.scrollTop += diff;
      schedule();
    }
  });

  function measure(node: HTMLElement) {
    ro.observe(node);
    return { destroy: () => ro.unobserve(node) };
  }

  $effect(() => {
    if (!host) return;
    const own = new ResizeObserver(() => { if (host) viewport = host.clientHeight; });
    own.observe(host);
    viewport = host.clientHeight;
    return () => { own.disconnect(); ro.disconnect(); cancelAnimationFrame(raf); };
  });
</script>

<div class="vlist" bind:this={host} onscroll={onScroll} role="log" aria-label={label} aria-live="polite">
  <div style:height="{top}px" aria-hidden="true"></div>
  {#each items.slice(win.start, win.end) as item (key(item))}
    <div data-k={key(item)} use:measure>{@render row(item)}</div>
  {/each}
  <div style:height="{bottom}px" aria-hidden="true"></div>
</div>

<style>
  .vlist {
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    overscroll-behavior: contain;
    contain: strict;
    scrollbar-width: thin;
    scrollbar-color: var(--surface-2) transparent;
  }
</style>
