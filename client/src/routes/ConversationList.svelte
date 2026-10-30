<script lang="ts">
  import { app, go } from "../lib/app.svelte";
  import { connText, initials, stateLabel, timeShort } from "../lib/format";
  import Header from "../lib/ui/Header.svelte";
  import Icon from "../lib/ui/Icon.svelte";
</script>

<section class="list">
  <Header title="ancord">
    {#snippet right()}
      <button class="icon-btn" aria-label="Kontakty" onclick={() => go("contacts")}><Icon name="users" /></button>
      <button class="icon-btn" aria-label="Nastavení" onclick={() => go("settings")}><Icon name="settings" /></button>
    {/snippet}
  </Header>

  <div class="chips">
    <span class="chip dim" role="status"><span class="dot {app.conn}"></span>{connText(app.conn, app.connIssue)}</span>
    <button class="chip dim" onclick={() => go("me")}>Moje adresa</button>
  </div>

  {#if app.conversations.length === 0}
    <div class="empty">
      <Icon name="chat" size={40} />
      <p>Zatím žádné konverzace.</p>
      <button class="btn primary" onclick={() => go("add")}><Icon name="plus" size={20} />Přidat kontakt</button>
    </div>
  {:else}
    <ul>
      {#each app.conversations as c (c.id)}
        <li>
          <button class="row" class:active={app.chatId === c.id} onclick={() => go("chat", c.id)}>
            <span class="avatar" aria-hidden="true">{initials(c.name)}</span>
            <span class="text">
              <span class="top">
                <span class="name">{c.name}</span>
                <span class="when muted">{timeShort(c.lastTs)}</span>
              </span>
              <span class="bottom">
                <span class="last muted">
                  {#if c.lastState && c.lastState !== "delivered"}<em>{stateLabel[c.lastState]} · </em>{/if}{c.lastText}
                </span>
                {#if c.unread > 0}<span class="badge" aria-label="{c.unread} nepřečtených">{c.unread}</span>{/if}
              </span>
            </span>
          </button>
        </li>
      {/each}
    </ul>
    <div class="fab-row"><button class="btn primary" onclick={() => go("add")}><Icon name="plus" size={20} />Nový kontakt</button></div>
  {/if}
</section>

<style>
  .list { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .chips { display: flex; gap: var(--space-2); padding: 0 var(--space-4) var(--space-3); flex-wrap: wrap; }
  ul { list-style: none; margin: 0; padding: 0 var(--space-2); flex: 1; overflow-y: auto; min-height: 0; }
  .row {
    display: flex; gap: var(--space-3); align-items: center;
    width: 100%; padding: var(--space-3); border-radius: var(--r-md); text-align: left;
  }
  .row:hover, .row.active { background: var(--surface-hover); }
  .avatar {
    flex: none; width: 46px; height: 46px; display: grid; place-items: center;
    border-radius: 50%; background: var(--surface-2); font-weight: 600; font-size: var(--fs-md);
  }
  .text { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .top, .bottom { display: flex; justify-content: space-between; gap: var(--space-2); align-items: center; }
  .name { font-weight: 600; font-size: var(--fs-lg); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .when { font-size: var(--fs-sm); flex: none; }
  .last { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: var(--fs-md); }
  .last em { font-style: normal; color: var(--warn); }
  .badge {
    flex: none; min-width: 22px; height: 22px; padding: 0 6px; display: grid; place-items: center;
    border-radius: var(--r-pill); background: var(--accent); color: var(--on-accent); font-size: var(--fs-sm); font-weight: 600;
  }
  .empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: var(--space-3); color: var(--muted); }
  .empty p { margin: 0; }
  .fab-row { display: flex; justify-content: center; padding: var(--space-3) var(--space-4) calc(var(--space-4) + var(--safe-bottom)); }
</style>
