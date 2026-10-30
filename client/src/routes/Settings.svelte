<script lang="ts">
  import { app, go, saveSettings, lockNow } from "../lib/app.svelte";
  import type { ThemeChoice } from "../lib/types";
  import Header from "../lib/ui/Header.svelte";
  import Icon from "../lib/ui/Icon.svelte";

  const themes: { v: ThemeChoice; l: string }[] = [
    { v: "system", l: "Systém" }, { v: "dark", l: "Tmavý" }, { v: "light", l: "Světlý" },
  ];
  const locks = [
    { v: 0, l: "Nikdy" }, { v: 60, l: "1 min" }, { v: 300, l: "5 min" }, { v: 900, l: "15 min" },
  ];
</script>

<section class="page">
  <Header title="Nastavení" onback={() => go("list")} />
  {#if app.settings}
    {@const s = app.settings}
    <div class="body">
      <div class="group">
        <h2>Vzhled</h2>
        <div class="seg" role="radiogroup" aria-label="Téma">
          {#each themes as t (t.v)}
            <button class="chip" class:on={s.theme === t.v} role="radio" aria-checked={s.theme === t.v} onclick={() => void saveSettings({ theme: t.v })}>{t.l}</button>
          {/each}
        </div>
      </div>

      <div class="group">
        <h2>Soukromí</h2>
        <label class="toggle card">
          <input type="checkbox" checked={s.enhancedProtection} onchange={(e) => void saveSettings({ enhancedProtection: e.currentTarget.checked })} />
          <span><strong>Zvýšená ochrana</strong><span class="muted">Skryje obsah před snímáním obrazovky a čtečkami, blokuje překryvná okna. Čtečky obrazovky pak nemusí fungovat.</span></span>
        </label>
        <label class="toggle card">
          <input type="checkbox" checked={s.notificationPreview} onchange={(e) => void saveSettings({ notificationPreview: e.currentTarget.checked })} />
          <span><strong>Náhled v oznámení</strong><span class="muted">Výchozí oznámení je jen „Nová zpráva“, bez odesílatele a obsahu.</span></span>
        </label>
        <div>
          <p class="lbl">Zamknout po nečinnosti</p>
          <div class="seg" role="radiogroup" aria-label="Zamknout po nečinnosti">
            {#each locks as l (l.v)}
              <button class="chip" class:on={s.lockAfterSecs === l.v} role="radio" aria-checked={s.lockAfterSecs === l.v} onclick={() => void saveSettings({ lockAfterSecs: l.v })}>{l.l}</button>
            {/each}
          </div>
        </div>
      </div>

      <div class="group">
        <h2>Spojení</h2>
        <label class="toggle card">
          <input type="checkbox" checked={s.keepConnection} onchange={(e) => void saveSettings({ keepConnection: e.currentTarget.checked })} />
          <span><strong>Trvalé spojení</strong><span class="muted">Pro Android bez UnifiedPush distributora. Drží spojení na pozadí s trvalým oznámením, stojí baterii.</span></span>
        </label>
      </div>

      <button class="btn" onclick={() => void lockNow()}><Icon name="lock" size={20} />Zamknout teď</button>
      <p class="muted small">Klient neposílá žádnou telemetrii a nekontroluje aktualizace. Mluví jen s Workerem a nody.</p>
    </div>
  {/if}
</section>

<style>
  .page { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .body { overflow-y: auto; padding: var(--space-2) var(--space-4) calc(var(--space-5) + var(--safe-bottom)); display: flex; flex-direction: column; gap: var(--space-5); max-width: 560px; width: 100%; margin: 0 auto; }
  .group { display: flex; flex-direction: column; gap: var(--space-3); }
  h2 { margin: 0; font-size: var(--fs-sm); text-transform: uppercase; letter-spacing: 0.06em; color: var(--muted); font-weight: 600; }
  .lbl { margin: 0 0 var(--space-2); }
  .seg { display: flex; gap: var(--space-2); flex-wrap: wrap; }
  .seg .chip { min-height: 38px; font-size: var(--fs-md); }
  .seg .chip.on { background: var(--surface-2); border-color: var(--accent); }
  .toggle { display: flex; gap: var(--space-3); padding: var(--space-4); align-items: flex-start; }
  .toggle input { margin-top: 4px; width: 18px; height: 18px; accent-color: var(--accent); flex: none; }
  .toggle span { display: flex; flex-direction: column; gap: 2px; }
  .small { font-size: var(--fs-sm); margin: 0; }
</style>
