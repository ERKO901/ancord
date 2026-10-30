<script lang="ts">
  import { app, go, markVerified, deleteConversation, errText } from "../lib/app.svelte";
  import { call } from "../lib/ipc";
  import { shortAddress } from "../lib/format";
  import Header from "../lib/ui/Header.svelte";
  import Icon from "../lib/ui/Icon.svelte";

  const contact = $derived(app.contacts.find((c) => c.id === app.contactId));
  let safety = $state("");
  let confirmDelete = $state(false);

  $effect(() => {
    const id = app.contactId;
    safety = "";
    confirmDelete = false;
    if (id) call("contact_safety_number", { id }).then((s) => { if (app.contactId === id) safety = s; }, (e) => { app.error = errText(e); });
  });
</script>

<section class="page">
  <Header title={contact?.name ?? "Kontakt"} onback={() => go("contacts")} />
  {#if contact}
    <div class="body">
      <div class="card block">
        <h2>Bezpečnostní číslo</h2>
        <p class="num mono selectable" aria-label="Bezpečnostní číslo">{safety}</p>
        <p class="muted small">Obě strany musí vidět totéž číslo. Porovnejte ho osobně nebo telefonem, ne přes ancord.</p>
        {#if contact.verified}
          <span class="chip ok"><Icon name="shield" size={18} />Ověřeno</span>
        {:else}
          <button class="btn" onclick={() => void markVerified(contact.id)}><Icon name="check" size={20} />Čísla souhlasí</button>
        {/if}
      </div>
      <div class="card block">
        <h2>Adresa</h2>
        <p class="mono small selectable wrap">{contact.address}</p>
        <span class="muted small">{shortAddress(contact.address)}</span>
      </div>
      <div class="actions">
        <button class="btn primary" onclick={() => go("chat", contact.id)}><Icon name="chat" size={20} />Napsat zprávu</button>
        {#if confirmDelete}
          <button class="btn danger" onclick={() => void deleteConversation(contact.id)}>Opravdu smazat historii?</button>
        {:else}
          <button class="btn danger" onclick={() => (confirmDelete = true)}><Icon name="trash" size={20} />Smazat historii</button>
        {/if}
      </div>
    </div>
  {/if}
</section>

<style>
  .page { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .body { overflow-y: auto; padding: var(--space-2) var(--space-4) calc(var(--space-5) + var(--safe-bottom)); display: flex; flex-direction: column; gap: var(--space-4); max-width: 560px; width: 100%; margin: 0 auto; }
  .block { padding: var(--space-4); display: flex; flex-direction: column; gap: var(--space-3); align-items: flex-start; }
  h2 { margin: 0; font-size: var(--fs-md); font-weight: 600; }
  .num { margin: 0; font-size: 1.25rem; line-height: 1.8; letter-spacing: 0.04em; word-spacing: 0.5em; }
  .small { font-size: var(--fs-sm); margin: 0; }
  .wrap { overflow-wrap: anywhere; }
  .ok { color: var(--ok); }
  .actions { display: flex; flex-direction: column; gap: var(--space-2); }
</style>
