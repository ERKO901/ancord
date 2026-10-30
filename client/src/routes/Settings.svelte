<script lang="ts">
  import { app, go, saveSettings, lockNow } from "../lib/app.svelte";
  import type { ThemeChoice } from "../lib/types";
  import Header from "../lib/ui/Header.svelte";
  import Icon from "../lib/ui/Icon.svelte";
  import Switch from "../lib/ui/Switch.svelte";

  const themes: { v: ThemeChoice; l: string }[] = [
    { v: "system", l: "Systém" }, { v: "dark", l: "Tmavý" }, { v: "light", l: "Světlý" },
  ];
  const locks = [
    { v: 0, l: "Nikdy" }, { v: 60, l: "1 min" }, { v: 300, l: "5 min" }, { v: 900, l: "15 min" },
  ];
</script>

<section class="page">
  <Header title="Nastavení" large />
  {#if app.settings}
    {@const s = app.settings}
    <div class="body">
      <div>
        <h2 class="group-title">Vzhled</h2>
        <div class="list-card pad">
          <p class="lbl">Téma</p>
          <div class="seg" role="radiogroup" aria-label="Téma">
            {#each themes as t (t.v)}
              <button class="chip" class:on={s.theme === t.v} role="radio" aria-checked={s.theme === t.v} onclick={() => void saveSettings({ theme: t.v })}>{t.l}</button>
            {/each}
          </div>
        </div>
      </div>

      <div>
        <h2 class="group-title">Soukromí a zabezpečení</h2>
        <div class="list-card">
          <div class="setting">
            <span class="t"><strong>Zvýšená ochrana</strong><span>Skryje obsah před snímáním obrazovky a čtečkami, blokuje překryvná okna. Čtečky obrazovky pak nemusí fungovat.</span></span>
            <Switch label="Zvýšená ochrana" checked={s.enhancedProtection} onchange={(v) => void saveSettings({ enhancedProtection: v })} />
          </div>
          <div class="setting">
            <span class="t"><strong>Náhled v oznámení</strong><span>Výchozí oznámení je jen „Nová zpráva“, bez odesílatele a obsahu.</span></span>
            <Switch label="Náhled v oznámení" checked={s.notificationPreview} onchange={(v) => void saveSettings({ notificationPreview: v })} />
          </div>
          <div class="setting stack">
            <span class="t"><strong>Zamknout po nečinnosti</strong></span>
            <div class="seg" role="radiogroup" aria-label="Zamknout po nečinnosti">
              {#each locks as l (l.v)}
                <button class="chip" class:on={s.lockAfterSecs === l.v} role="radio" aria-checked={s.lockAfterSecs === l.v} onclick={() => void saveSettings({ lockAfterSecs: l.v })}>{l.l}</button>
              {/each}
            </div>
          </div>
          <button class="setting" onclick={() => void lockNow()}>
            <span class="lead"><Icon name="lock" size={20} /></span>
            <span class="t"><strong>Zamknout teď</strong><span>Obsah zpráv zmizí z paměti aplikace.</span></span>
            <Icon name="chevron" size={20} />
          </button>
        </div>
      </div>

      <div>
        <h2 class="group-title">Spojení</h2>
        <div class="list-card">
          <div class="setting">
            <span class="t"><strong>Trvalé spojení</strong><span>Pro Android bez UnifiedPush distributora. Drží spojení na pozadí s trvalým oznámením, stojí baterii.</span></span>
            <Switch label="Trvalé spojení" checked={s.keepConnection} onchange={(v) => void saveSettings({ keepConnection: v })} />
          </div>
        </div>
      </div>

      <div>
        <h2 class="group-title">O aplikaci</h2>
        <div class="list-card">
          <button class="setting" onclick={() => go("about")}>
            <span class="lead"><Icon name="info" size={20} /></span>
            <span class="t"><strong>O ancordu</strong><span>Verze, soukromí a jak to funguje</span></span>
            <Icon name="chevron" size={20} />
          </button>
        </div>
      </div>
    </div>
  {/if}
</section>

<style>
  .page { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .body { overflow-y: auto; padding: var(--space-2) var(--space-4) var(--space-5); display: flex; flex-direction: column; gap: var(--space-5); max-width: 560px; width: 100%; margin: 0 auto; }
  .pad { padding: var(--space-4); }
  .list-card.pad > * { border: 0; }
  .lbl { margin: 0 0 var(--space-2); font-weight: 500; }
  .seg { display: flex; gap: var(--space-2); flex-wrap: wrap; }
  .seg .chip { min-height: 38px; font-size: var(--fs-md); }
  .seg .chip.on { background: var(--accent-soft); border-color: var(--accent); color: var(--accent); font-weight: 600; }
  .stack { flex-direction: column; align-items: flex-start; }
</style>
