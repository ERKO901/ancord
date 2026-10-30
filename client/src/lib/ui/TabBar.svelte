<script lang="ts">
  import { app, go, tabOf, totalUnread, type View } from "../app.svelte";
  import Icon from "./Icon.svelte";

  const tabs = [
    { view: "list", label: "Chaty", icon: "bubble" },
    { view: "contacts", label: "Kontakty", icon: "users" },
    { view: "me", label: "Profil", icon: "user" },
    { view: "settings", label: "Nastavení", icon: "settings" },
  ] as const satisfies readonly { view: View; label: string; icon: string }[];

  const current = $derived(tabOf(app.view));
  const unread = $derived(totalUnread());
</script>

<nav class="tabs" aria-label="Hlavní navigace">
  {#each tabs as t (t.view)}
    <button class="tab" class:on={current === t.view} aria-current={current === t.view ? "page" : undefined} onclick={() => go(t.view)}>
      <span class="ico">
        <Icon name={t.icon} size={24} />
        {#if t.view === "list" && unread > 0}<span class="dotbadge" aria-label="{unread} nepřečtených"></span>{/if}
      </span>
      <span class="lbl">{t.label}</span>
    </button>
  {/each}
</nav>

<style>
  .tabs {
    display: flex;
    flex: none;
    height: calc(var(--tabbar) + var(--safe-bottom));
    padding-bottom: var(--safe-bottom);
    background: var(--surface);
    border-top: 1px solid var(--border);
  }
  .tab {
    flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 2px;
    color: var(--muted); font-size: 0.75rem; font-weight: 500;
  }
  .ico {
    position: relative; display: grid; place-items: center;
    width: 56px; height: 30px; border-radius: var(--r-pill); transition: background 0.15s, color 0.15s;
  }
  .tab.on { color: var(--text); }
  .tab.on .ico { background: var(--accent-soft); color: var(--accent); }
  .tab:hover:not(.on) .ico { background: var(--surface-hover); }
  .dotbadge {
    position: absolute; top: 3px; right: 13px; width: 9px; height: 9px; border-radius: 50%;
    background: var(--accent); border: 2px solid var(--surface);
  }
</style>
