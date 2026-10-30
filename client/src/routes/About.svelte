<script lang="ts">
  import { go } from "../lib/app.svelte";
  import { version } from "../../package.json";
  import Header from "../lib/ui/Header.svelte";
  import Icon from "../lib/ui/Icon.svelte";
  import Logo from "../lib/ui/Logo.svelte";

  const points = [
    { icon: "lock", t: "Šifrováno od konce ke konci", d: "Zprávy čtete jen vy a váš kontakt. Uzly sítě vidí jen šifrovaná data." },
    { icon: "users", t: "Síť bez jedné firmy", d: "Uzly provozuje kdokoli. Žádný centrální server si nemůže přečíst vaše zprávy." },
    { icon: "shield", t: "Klíče zůstávají u vás", d: "Soukromé klíče nikdy neopustí zařízení a nedostanou se ani do uživatelského rozhraní." },
    { icon: "shieldcheck", t: "Žádná telemetrie", d: "Aplikace neposílá statistiky, nehlásí pády a nekontroluje aktualizace. Mluví jen s Workerem a uzly." },
  ] as const;
</script>

<section class="page">
  <Header title="O ancordu" onback={() => go("settings")} />
  <div class="body">
    <div class="hero">
      <Logo size={88} />
      <h2>ancord</h2>
      <p class="muted">Verze {version}</p>
      <p class="tag">Soukromé zprávy v decentralizované síti.</p>
    </div>
    <ul class="list-card">
      {#each points as p (p.t)}
        <li class="setting">
          <span class="lead"><Icon name={p.icon} size={20} /></span>
          <span class="t"><strong>{p.t}</strong><span>{p.d}</span></span>
        </li>
      {/each}
    </ul>
    <p class="muted small">Ztráta zařízení znamená ztrátu identity. Záloha a více zařízení zatím nejsou.</p>
  </div>
</section>

<style>
  .page { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .body { overflow-y: auto; padding: var(--space-2) var(--space-4) calc(var(--space-5) + var(--safe-bottom)); display: flex; flex-direction: column; gap: var(--space-4); max-width: 560px; width: 100%; margin: 0 auto; }
  .hero { display: flex; flex-direction: column; align-items: center; gap: 2px; padding: var(--space-4) 0; text-align: center; }
  .hero h2 { margin: var(--space-3) 0 0; font-size: var(--fs-2xl); letter-spacing: -0.02em; }
  .hero p { margin: 0; }
  .tag { margin-top: var(--space-2) !important; }
  ul { list-style: none; margin: 0; padding: 0; }
  .small { font-size: var(--fs-sm); text-align: center; margin: 0; }
</style>
