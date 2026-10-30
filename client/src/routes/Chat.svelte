<script lang="ts">
  import { app, go, loadOlder, contactName } from "../lib/app.svelte";
  import { connText, dayLabel, sameDay, timeFull, timeShort, stateLabel } from "../lib/format";
  import type { Message } from "../lib/types";
  import Avatar from "../lib/ui/Avatar.svelte";
  import Header from "../lib/ui/Header.svelte";
  import Icon from "../lib/ui/Icon.svelte";
  import VirtualList from "../lib/ui/VirtualList.svelte";
  import Composer from "./Composer.svelte";

  const name = $derived(contactName(app.chatId));
  const contact = $derived(app.contacts.find((c) => c.id === app.chatId));
  const typing = $derived(app.chatId ? app.typing[app.chatId] === true : false);
  const subtitle = $derived(typing ? "píše…" : connText(app.conn, app.connIssue));

  // Den se ukáže nad první zprávou dne; seskupení: zprávy téhož autora do 2 minut za sebou drží u sebe.
  const marks = $derived.by(() => {
    const m = new Map<string, { day: boolean; first: boolean; last: boolean }>();
    const xs = app.messages;
    xs.forEach((x, i) => {
      const p = xs[i - 1];
      const n = xs[i + 1];
      const joined = (a: Message | undefined, b: Message) => !!a && a.mine === b.mine && sameDay(a.ts, b.ts) && Math.abs(b.ts - a.ts) < 120_000;
      m.set(x.id, { day: !p || !sameDay(p.ts, x.ts), first: !joined(p, x), last: !n || !joined(x, n) });
    });
    return m;
  });
</script>

<section class="chat">
  <Header title={name} {subtitle} onback={() => go("list")} hideBackOnWide>
    {#snippet lead()}<Avatar {name} size={40} />{/snippet}
    {#snippet right()}
      <button class="icon-btn" aria-label="Detail kontaktu" onclick={() => app.chatId && go("contact", app.chatId)}>
        <Icon name={contact?.verified ? "shieldcheck" : "shield"} />
      </button>
    {/snippet}
  </Header>

  <div class="list">
    {#if app.messages.length === 0}
      <div class="empty">
        <div class="art"><Icon name="lock" size={30} /></div>
        <h2>Soukromá konverzace</h2>
        <p>Zprávy jsou šifrované mezi vámi a kontaktem {name}. Napište první.</p>
        {#if contact && !contact.verified}
          <button class="btn" onclick={() => app.chatId && go("contact", app.chatId)}><Icon name="shield" size={20} />Ověřit bezpečnostní číslo</button>
        {/if}
      </div>
    {:else}
      <VirtualList items={app.messages} key={(m: Message) => m.id} estimate={64} onTop={() => void loadOlder()} label="Zprávy">
        {#snippet row(m: Message)}
          {@const mk = marks.get(m.id)}
          {#if mk?.day}<div class="day"><span>{dayLabel(m.ts)}</span></div>{/if}
          <article class="msg" class:mine={m.mine} class:first={mk?.first} class:last={mk?.last}>
            <div class="body selectable" class:out={m.mine} class:tail={mk?.last}>{m.text}</div>
            {#if mk?.last}
              <div class="meta" title={timeFull(m.ts)}>
                {timeShort(m.ts)}
                {#if m.mine}
                  {#if m.state === "delivered"}<Icon name="check2" size={14} />{:else if m.state === "unknown"}<Icon name="alert" size={14} />{:else}<Icon name="clock" size={14} />{/if}
                  {#if m.state !== "delivered"}<span>{stateLabel[m.state]}</span>{/if}
                {/if}
              </div>
            {/if}
          </article>
        {/snippet}
      </VirtualList>
    {/if}
  </div>

  <Composer />
</section>

<style>
  .chat { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .list { flex: 1; min-height: 0; }
  .empty { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: var(--space-3); padding: var(--space-5); text-align: center; }
  .empty h2 { margin: 0; font-size: var(--fs-lg); }
  .empty p { margin: 0; color: var(--muted); max-width: 300px; }
  .art { width: 72px; height: 72px; display: grid; place-items: center; border-radius: 50%; background: var(--accent-soft); color: var(--accent); }

  .day { display: flex; justify-content: center; padding: var(--space-4) 0 var(--space-2); }
  .day span { padding: 4px 12px; border-radius: var(--r-pill); background: var(--surface); border: 1px solid var(--border); font-size: var(--fs-sm); color: var(--muted); text-transform: capitalize; }

  .msg { padding: 1px var(--space-3); display: flex; flex-direction: column; align-items: flex-start; }
  .msg.first { padding-top: var(--space-2); }
  .msg.mine { align-items: flex-end; }
  .body {
    max-width: min(82%, 600px);
    text-align: left;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    font-size: var(--fs-lg);
    line-height: 1.45;
    padding: 9px 14px;
    border-radius: 20px;
    background: var(--bubble-in);
  }
  .body.out { background: var(--bubble-out); color: var(--on-bubble-out); }
  .body.tail { border-bottom-left-radius: 6px; }
  .body.out.tail { border-bottom-left-radius: 20px; border-bottom-right-radius: 6px; }
  .meta {
    display: flex; align-items: center; gap: 4px;
    margin: 2px 0 var(--space-1); padding: 0 var(--space-2);
    font-size: 0.75rem; color: var(--muted);
  }
</style>
