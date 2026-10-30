<script lang="ts">
  import { app, go, loadOlder, contactName } from "../lib/app.svelte";
  import { timeFull, timeShort, stateLabel } from "../lib/format";
  import type { Message } from "../lib/types";
  import Header from "../lib/ui/Header.svelte";
  import Icon from "../lib/ui/Icon.svelte";
  import VirtualList from "../lib/ui/VirtualList.svelte";
  import Composer from "./Composer.svelte";

  const name = $derived(contactName(app.chatId));
  const typing = $derived(app.chatId ? app.typing[app.chatId] === true : false);
</script>

<section class="chat">
  <Header title={name} onback={() => go("list")} hideBackOnWide>
    {#snippet right()}
      <button class="icon-btn" aria-label="Detail kontaktu" onclick={() => app.chatId && go("contact", app.chatId)}><Icon name="shield" /></button>
    {/snippet}
  </Header>

  <div class="list">
    {#if app.messages.length === 0}
      <p class="empty muted">Zatím žádné zprávy. Zprávy se zobrazují jako čistý text a jsou šifrované mezi vámi a kontaktem.</p>
    {:else}
      <VirtualList items={app.messages} key={(m: Message) => m.id} estimate={56} onTop={() => void loadOlder()} label="Zprávy">
        {#snippet row(m: Message)}
          <article class="msg" class:mine={m.mine}>
            <div class="body selectable" class:bubble={m.mine}>{m.text}</div>
            <div class="meta" title={timeFull(m.ts)}>
              {timeShort(m.ts)}
              {#if m.mine}
                {#if m.state === "delivered"}<Icon name="check2" size={14} />{:else if m.state === "unknown"}<Icon name="alert" size={14} />{:else}<Icon name="clock" size={14} />{/if}
                {#if m.state !== "delivered"}<span>{stateLabel[m.state]}</span>{/if}
              {/if}
            </div>
          </article>
        {/snippet}
      </VirtualList>
    {/if}
  </div>

  <div class="typing" aria-live="polite">{#if typing}{name} píše…{/if}</div>
  <Composer />
</section>

<style>
  .chat { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .list { flex: 1; min-height: 0; }
  .empty { margin: var(--space-5); text-align: center; }
  .msg { padding: var(--space-2) var(--space-4); display: flex; flex-direction: column; align-items: flex-start; }
  .msg.mine { align-items: flex-end; }
  .body {
    max-width: min(86%, 640px);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    font-size: var(--fs-lg);
    line-height: 1.5;
  }
  .bubble {
    background: var(--surface-2);
    padding: var(--space-3) var(--space-4);
    border-radius: 20px 20px 6px 20px;
  }
  .meta {
    display: flex; align-items: center; gap: 4px;
    margin-top: 2px; padding: 0 var(--space-1);
    font-size: var(--fs-sm); color: var(--muted);
  }
  .typing { min-height: 22px; padding: 0 var(--space-5); font-size: var(--fs-sm); color: var(--muted); }
</style>
