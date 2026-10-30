<script lang="ts">
  import { app, go } from "../lib/app.svelte";
  import { byName, fold, shortAddress } from "../lib/format";
  import Avatar from "../lib/ui/Avatar.svelte";
  import Header from "../lib/ui/Header.svelte";
  import Icon from "../lib/ui/Icon.svelte";

  let query = $state("");

  // Abecední sekce; písmeno bez diakritiky, ostatní do „#“.
  const groups = $derived.by(() => {
    const q = fold(query.trim());
    const xs = app.contacts.filter((c) => !q || fold(c.name).includes(q) || fold(c.address).includes(q)).sort(byName);
    const out: { letter: string; items: typeof xs }[] = [];
    for (const c of xs) {
      const first = fold(c.name).charAt(0).toUpperCase();
      const letter = /[A-Z]/.test(first) ? first : "#";
      const last = out[out.length - 1];
      if (last?.letter === letter) last.items.push(c);
      else out.push({ letter, items: [c] });
    }
    return out;
  });
</script>

<section class="page">
  <Header title="Kontakty" large>
    {#snippet right()}<button class="icon-btn" aria-label="Přidat kontakt" onclick={() => go("add")}><Icon name="userplus" /></button>{/snippet}
  </Header>
  {#if app.contacts.length === 0}
    <div class="empty">
      <div class="art"><Icon name="users" size={34} /></div>
      <h2>Žádné kontakty</h2>
      <p>Kontakt přidáte podle adresy <span class="mono">ancord:…</span> nebo naskenováním QR kódu.</p>
      <button class="btn primary" onclick={() => go("add")}><Icon name="userplus" size={20} />Přidat kontakt</button>
    </div>
  {:else}
    <label class="search">
      <Icon name="search" size={20} />
      <input type="search" placeholder="Hledat kontakty" aria-label="Hledat kontakty" bind:value={query} autocomplete="off" spellcheck="false" />
    </label>
    <div class="scroll">
      {#each groups as g (g.letter)}
        <h2 class="group-title">{g.letter}</h2>
        <ul class="list-card">
          {#each g.items as c (c.id)}
            <li>
              <button class="setting" onclick={() => go("contact", c.id)}>
                <Avatar name={c.name} size={42} />
                <span class="t">
                  <strong>{c.name}</strong>
                  <span class="mono">{shortAddress(c.address)}</span>
                </span>
                {#if c.verified}<span class="ok" title="Ověřeno"><Icon name="shieldcheck" size={22} /></span>
                {:else}<span class="unver">Neověřeno</span>{/if}
              </button>
            </li>
          {/each}
        </ul>
      {:else}
        <p class="muted none">Nic jsme nenašli.</p>
      {/each}
    </div>
  {/if}
</section>

<style>
  .page { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .scroll { overflow-y: auto; padding: 0 var(--space-4) var(--space-5); min-height: 0; flex: 1; }
  .scroll .group-title { margin-top: var(--space-3); }
  ul { list-style: none; margin: 0; padding: 0; }
  .ok { color: var(--ok); display: grid; }
  .unver { font-size: var(--fs-sm); color: var(--warn); flex: none; }
  .none { text-align: center; padding: var(--space-5); margin: 0; }
  .empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: var(--space-3); padding: var(--space-5); text-align: center; }
  .empty h2 { margin: 0; font-size: var(--fs-xl); }
  .empty p { margin: 0; color: var(--muted); max-width: 320px; }
  .art { width: 88px; height: 88px; display: grid; place-items: center; border-radius: 50%; background: var(--accent-soft); color: var(--accent); }
</style>
