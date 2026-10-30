<script lang="ts">
  import { app, go } from "../lib/app.svelte";
  import { connText, fold, stateLabel, timeShort } from "../lib/format";
  import Avatar from "../lib/ui/Avatar.svelte";
  import Header from "../lib/ui/Header.svelte";
  import Icon from "../lib/ui/Icon.svelte";
  import Logo from "../lib/ui/Logo.svelte";

  let query = $state("");
  let onlyUnread = $state(false);

  const shown = $derived.by(() => {
    const q = fold(query.trim());
    return app.conversations.filter((c) => {
      if (onlyUnread && c.unread === 0) return false;
      return !q || fold(c.name).includes(q) || fold(c.lastText).includes(q);
    });
  });
  const unreadCount = $derived(app.conversations.filter((c) => c.unread > 0).length);
</script>

<section class="list">
  <Header title="ancord" subtitle={connText(app.conn, app.connIssue)} large>
    {#snippet lead()}<Logo size={36} />{/snippet}
    {#snippet right()}
      <span class="dot {app.conn}" aria-hidden="true"></span>
    {/snippet}
  </Header>

  {#if app.conn === "offline"}
    <div class="banner"><Icon name="alert" size={18} />Zprávy se odešlou, až se aplikace připojí.</div>
  {/if}

  {#if app.conversations.length === 0}
    <div class="empty">
      <div class="art"><Logo size={72} tile={false} /></div>
      <h2>Vítejte v ancordu</h2>
      <p>Zatím tu nejsou žádné konverzace. Přidejte kontakt podle adresy nebo QR kódu a napište první zprávu.</p>
      <button class="btn primary" onclick={() => go("add")}><Icon name="userplus" size={20} />Přidat kontakt</button>
      <button class="btn" onclick={() => go("me")}><Icon name="qr" size={20} />Zobrazit moji adresu</button>
    </div>
  {:else}
    <label class="search">
      <Icon name="search" size={20} />
      <input type="search" placeholder="Hledat v konverzacích" aria-label="Hledat v konverzacích" bind:value={query} autocomplete="off" spellcheck="false" />
    </label>
    <div class="filters" role="group" aria-label="Filtr">
      <button class="chip" class:on={!onlyUnread} aria-pressed={!onlyUnread} onclick={() => (onlyUnread = false)}>Vše</button>
      <button class="chip" class:on={onlyUnread} aria-pressed={onlyUnread} onclick={() => (onlyUnread = true)}>Nepřečtené{unreadCount > 0 ? ` · ${unreadCount}` : ""}</button>
    </div>
    {#if shown.length === 0}
      <p class="none muted">{onlyUnread && !query ? "Všechno máte přečtené." : "Nic jsme nenašli."}</p>
    {:else}
      <ul>
        {#each shown as c (c.id)}
          <li>
            <button class="row" class:active={app.chatId === c.id} onclick={() => go("chat", c.id)}>
              <Avatar name={c.name} size={50} />
              <span class="text">
                <span class="top">
                  <span class="name" class:strong={c.unread > 0}>{c.name}</span>
                  <span class="when" class:accent={c.unread > 0}>{timeShort(c.lastTs)}</span>
                </span>
                <span class="bottom">
                  <span class="last" class:strong={c.unread > 0}>
                    {#if c.lastState === "delivered"}<span class="tick"><Icon name="check2" size={15} /></span>
                    {:else if c.lastState}<em>{stateLabel[c.lastState]} · </em>{/if}{c.lastText}
                  </span>
                  {#if c.unread > 0}<span class="badge" aria-label="{c.unread} nepřečtených">{c.unread}</span>{/if}
                </span>
              </span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
    <button class="fab" aria-label="Nová konverzace" onclick={() => go("add")}><Icon name="plus" size={26} /></button>
  {/if}
</section>

<style>
  .list { position: relative; display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .filters { display: flex; gap: var(--space-2); padding: 0 var(--space-4) var(--space-2); }
  .filters .chip { min-height: 34px; font-size: var(--fs-md); padding: 0 var(--space-3); color: var(--muted); }
  .filters .chip.on { background: var(--accent-soft); border-color: transparent; color: var(--accent); font-weight: 600; }
  ul { list-style: none; margin: 0; padding: 0 var(--space-2) 88px; flex: 1; overflow-y: auto; min-height: 0; }
  .row {
    display: flex; gap: var(--space-3); align-items: center;
    width: 100%; padding: var(--space-3); border-radius: var(--r-md); text-align: left;
  }
  .row:hover, .row.active { background: var(--surface-hover); }
  .text { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .top, .bottom { display: flex; justify-content: space-between; gap: var(--space-2); align-items: center; }
  .name { font-weight: 600; font-size: var(--fs-lg); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .when { font-size: var(--fs-sm); flex: none; color: var(--muted); }
  .when.accent { color: var(--accent); font-weight: 600; }
  .last { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--muted); }
  .last.strong { color: var(--text); }
  .last em { font-style: normal; color: var(--warn); }
  .tick { color: var(--accent); margin-right: 4px; vertical-align: -2px; }
  .badge {
    flex: none; min-width: 22px; height: 22px; padding: 0 6px; display: grid; place-items: center;
    border-radius: var(--r-pill); background: var(--accent); color: var(--on-accent); font-size: var(--fs-sm); font-weight: 600;
  }
  .none { text-align: center; padding: var(--space-5); margin: 0; }
  .empty {
    flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: var(--space-3);
    padding: var(--space-5); text-align: center;
  }
  .empty h2 { margin: 0; font-size: var(--fs-xl); }
  .empty p { margin: 0; color: var(--muted); max-width: 320px; }
  .art { width: 112px; height: 112px; display: grid; place-items: center; border-radius: 50%; background: var(--accent-soft); margin-bottom: var(--space-2); }
</style>
