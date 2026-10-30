<script lang="ts">
  import { app, go, markVerified, deleteConversation, renameContact, errText } from "../lib/app.svelte";
  import { call } from "../lib/ipc";
  import { shortAddress } from "../lib/format";
  import Avatar from "../lib/ui/Avatar.svelte";
  import Header from "../lib/ui/Header.svelte";
  import Icon from "../lib/ui/Icon.svelte";

  const contact = $derived(app.contacts.find((c) => c.id === app.contactId));
  let safety = $state("");
  let confirmDelete = $state(false);
  let copied = $state(false);
  let label = $state("");
  let editing = $state(false);

  $effect(() => {
    const id = app.contactId;
    safety = "";
    confirmDelete = false;
    editing = false;
    if (id) call("contact_safety_number", { id }).then((s) => { if (app.contactId === id) safety = s; }, (e) => { app.error = errText(e); });
  });

  async function saveLabel() {
    if (!contact) return;
    try {
      await renameContact(contact.id, label);
      editing = false;
    } catch (e) {
      app.error = errText(e);
    }
  }

  async function copyAddress() {
    if (!contact) return;
    try {
      await navigator.clipboard.writeText(contact.address);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch {
      app.error = "Adresu se nepodařilo zkopírovat";
    }
  }
</script>

<section class="page">
  <Header title="Kontakt" onback={() => go("contacts")} />
  {#if contact}
    <div class="body">
      <div class="hero">
        <Avatar name={contact.name} size={96} />
        <h2>{contact.name}</h2>
        {#if contact.peerName && contact.peerName !== contact.name}
          <p class="muted small">Sám sebe nazývá „{contact.peerName}“</p>
        {:else if !contact.peerName}
          <p class="muted small">Jméno uvidíte, jakmile vám odpoví.</p>
        {/if}
        {#if editing}
          <form class="rename" onsubmit={(e) => { e.preventDefault(); void saveLabel(); }}>
            <input class="field" type="text" maxlength="32" placeholder={contact.peerName ?? "Jméno kontaktu"} aria-label="Jméno kontaktu" bind:value={label} />
            <button class="btn primary" type="submit">Uložit</button>
          </form>
        {:else}
          <button class="btn small-btn" onclick={() => { label = contact.name; editing = true; }}><Icon name="edit" size={18} />Přejmenovat</button>
        {/if}
        {#if contact.verified}
          <span class="status ok"><Icon name="shieldcheck" size={18} />Ověřený kontakt</span>
        {:else}
          <span class="status warn"><Icon name="alert" size={18} />Zatím neověřeno</span>
        {/if}
        <button class="btn primary" onclick={() => go("chat", contact.id)}><Icon name="bubble" size={20} />Napsat zprávu</button>
      </div>

      <div class="card block">
        <h3>Bezpečnostní číslo</h3>
        <p class="num mono selectable" aria-label="Bezpečnostní číslo">{safety}</p>
        <p class="muted small">Obě strany musí vidět totéž číslo. Porovnejte ho osobně nebo telefonem, ne přes ancord. Tak poznáte, že vám nikdo nepodstrčil cizí klíč.</p>
        {#if !contact.verified}
          <button class="btn" onclick={() => void markVerified(contact.id)}><Icon name="check" size={20} />Čísla souhlasí</button>
        {/if}
      </div>

      <div class="card block">
        <h3>Adresa</h3>
        <p class="mono small selectable wrap">{contact.address}</p>
        <div class="row">
          <span class="muted small">{shortAddress(contact.address)}</span>
          <button class="btn small-btn" onclick={copyAddress}><Icon name="copy" size={18} />{copied ? "Zkopírováno" : "Kopírovat"}</button>
        </div>
      </div>

      {#if confirmDelete}
        <button class="btn danger" onclick={() => void deleteConversation(contact.id)}>Opravdu smazat historii?</button>
      {:else}
        <button class="btn danger" onclick={() => (confirmDelete = true)}><Icon name="trash" size={20} />Smazat historii</button>
      {/if}
    </div>
  {/if}
</section>

<style>
  .page { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .body { overflow-y: auto; padding: var(--space-2) var(--space-4) calc(var(--space-5) + var(--safe-bottom)); display: flex; flex-direction: column; gap: var(--space-4); max-width: 560px; width: 100%; margin: 0 auto; }
  .hero { display: flex; flex-direction: column; align-items: center; gap: var(--space-2); padding: var(--space-3) 0; }
  .hero h2 { margin: var(--space-2) 0 0; font-size: var(--fs-xl); }
  .status { display: inline-flex; align-items: center; gap: 6px; font-size: var(--fs-md); margin-bottom: var(--space-2); }
  .status.ok { color: var(--ok); }
  .status.warn { color: var(--warn); }
  .block { padding: var(--space-4); display: flex; flex-direction: column; gap: var(--space-3); align-items: flex-start; }
  h3 { margin: 0; font-size: var(--fs-md); font-weight: 600; }
  .num { margin: 0; font-size: 1.25rem; line-height: 1.8; letter-spacing: 0.04em; word-spacing: 0.5em; }
  .small { font-size: var(--fs-sm); margin: 0; }
  .wrap { overflow-wrap: anywhere; }
  .row { display: flex; align-items: center; justify-content: space-between; width: 100%; gap: var(--space-3); }
  .rename { display: flex; gap: var(--space-2); width: 100%; max-width: 360px; }
  .rename .field { flex: 1; min-width: 0; }
  .small-btn { min-height: 38px; padding: 0 var(--space-4); font-size: var(--fs-md); }
</style>
