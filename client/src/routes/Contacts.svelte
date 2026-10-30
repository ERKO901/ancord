<script lang="ts">
  import { app, go } from "../lib/app.svelte";
  import { initials, shortAddress } from "../lib/format";
  import Header from "../lib/ui/Header.svelte";
  import Icon from "../lib/ui/Icon.svelte";
</script>

<section class="page">
  <Header title="Kontakty" onback={() => go("list")}>
    {#snippet right()}<button class="icon-btn" aria-label="Přidat kontakt" onclick={() => go("add")}><Icon name="plus" /></button>{/snippet}
  </Header>
  {#if app.contacts.length === 0}
    <p class="muted empty">Žádné kontakty. Přidejte je podle adresy <span class="mono">ancord:…</span>.</p>
  {:else}
    <ul>
      {#each app.contacts as c (c.id)}
        <li>
          <button class="row" onclick={() => go("contact", c.id)}>
            <span class="avatar" aria-hidden="true">{initials(c.name)}</span>
            <span class="t">
              <span class="name">{c.name}</span>
              <span class="muted mono addr">{shortAddress(c.address)}</span>
            </span>
            {#if c.verified}<span class="ok" title="Ověřeno"><Icon name="shield" size={20} /></span>{/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .page { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  ul { list-style: none; margin: 0; padding: 0 var(--space-2); overflow-y: auto; }
  .row { display: flex; gap: var(--space-3); align-items: center; width: 100%; padding: var(--space-3); border-radius: var(--r-md); text-align: left; }
  .row:hover { background: var(--surface-hover); }
  .avatar { flex: none; width: 42px; height: 42px; display: grid; place-items: center; border-radius: 50%; background: var(--surface-2); font-weight: 600; }
  .t { flex: 1; display: flex; flex-direction: column; min-width: 0; }
  .name { font-weight: 600; font-size: var(--fs-lg); }
  .addr { font-size: var(--fs-sm); }
  .ok { color: var(--ok); }
  .empty { padding: var(--space-5); text-align: center; }
</style>
