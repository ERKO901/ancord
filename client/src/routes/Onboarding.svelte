<script lang="ts">
  import { app, createIdentity, errText } from "../lib/app.svelte";
  import Icon from "../lib/ui/Icon.svelte";
  import Logo from "../lib/ui/Logo.svelte";

  const slides = [
    { icon: "lock", title: "Zprávy jen pro vás", text: "Šifrování od konce ke konci. Obsah přečtete jen vy a váš kontakt, nikdo mezi vámi." },
    { icon: "users", title: "Síť bez jedné firmy", text: "Zprávy putují přes uzly, které provozuje kdokoli. Nikdo nemá nad sítí kontrolu sám." },
    { icon: "shield", title: "Klíče zůstávají u vás", text: "Soukromé klíče nikdy neopustí toto zařízení. Žádný účet, telefonní číslo ani e-mail." },
  ] as const;

  // 0 = uvítání, 1–3 = slajdy, 4 = vytvoření identity
  let step = $state(0);
  let usePass = $state(false);
  let pass = $state("");
  let pass2 = $state("");
  let busy = $state(false);

  const weak = $derived(usePass && (pass.length < 8 || pass !== pass2));
  const slide = $derived(step >= 1 && step <= slides.length ? slides[step - 1] : undefined);

  async function create() {
    busy = true;
    try {
      await createIdentity(usePass ? pass : undefined);
    } catch (e) {
      app.error = errText(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="wrap">
  {#if step === 0}
    <div class="hero">
      <Logo size={112} />
      <h1>ancord</h1>
      <p class="tag">Soukromé zprávy v decentralizované síti.</p>
    </div>
    <div class="foot">
      <button class="btn primary big" onclick={() => (step = 1)}>Začít</button>
    </div>
  {:else if slide}
    <div class="hero">
      <div class="art"><Icon name={slide.icon} size={56} /></div>
      <h1>{slide.title}</h1>
      <p class="muted">{slide.text}</p>
    </div>
    <div class="foot">
      <div class="dots" aria-hidden="true">
        {#each slides as _, i (i)}<span class:on={i === step - 1}></span>{/each}
      </div>
      <button class="btn primary big" onclick={() => (step += 1)}>Další</button>
      <button class="btn skip" onclick={() => (step = slides.length + 1)}>Přeskočit</button>
    </div>
  {:else}
    <div class="hero compact">
      <Logo size={64} />
      <h1>Vytvořte si identitu</h1>
      <p class="muted">Vygeneruje se na tomto zařízení za okamžik.</p>
    </div>
    <div class="card form">
      <div class="opt">
        <span>
          <strong>Chránit heslem</strong>
          <span class="muted">Na pozadí pak nejdou zprávy dešifrovat, oznámení bude jen „Nová zpráva“ a zprávy se vyzvednou po odemknutí.</span>
        </span>
        <button type="button" class="sw" role="switch" aria-checked={usePass} aria-label="Chránit heslem" onclick={() => (usePass = !usePass)}><span class="knob"></span></button>
      </div>
      {#if usePass}
        <input class="field" type="password" placeholder="Heslo (alespoň 8 znaků)" bind:value={pass} autocomplete="new-password" />
        <input class="field" type="password" placeholder="Heslo znovu" bind:value={pass2} autocomplete="new-password" />
      {/if}
      <p class="muted warn"><Icon name="alert" size={16} /> Ztráta zařízení znamená ztrátu identity. Záloha a více zařízení zatím nejsou.</p>
    </div>
    <div class="foot">
      <button class="btn primary big" onclick={create} disabled={busy || weak}>Vytvořit identitu</button>
      <button class="btn skip" onclick={() => (step = 0)}>Zpět</button>
    </div>
  {/if}
</section>

<style>
  .wrap {
    height: 100%; overflow-y: auto; display: flex; flex-direction: column; justify-content: space-between; gap: var(--space-5);
    padding: calc(var(--space-5) + var(--safe-top)) var(--space-5) calc(var(--space-5) + var(--safe-bottom)); max-width: 480px; margin: 0 auto;
  }
  .hero { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: var(--space-3); text-align: center; }
  .hero.compact { flex: none; padding-top: var(--space-4); }
  .hero h1 { margin: var(--space-3) 0 0; font-size: var(--fs-2xl); letter-spacing: -0.02em; }
  .hero p { margin: 0; max-width: 340px; font-size: var(--fs-lg); }
  .tag { color: var(--muted); }
  .art { width: 128px; height: 128px; display: grid; place-items: center; border-radius: 50%; background: var(--accent-soft); color: var(--accent); }
  .foot { display: flex; flex-direction: column; gap: var(--space-2); }
  .big { min-height: 52px; font-size: var(--fs-lg); }
  .skip { background: transparent; color: var(--muted); }
  .dots { display: flex; justify-content: center; gap: 8px; margin-bottom: var(--space-3); }
  .dots span { width: 8px; height: 8px; border-radius: 50%; background: var(--surface-2); transition: width 0.2s, background 0.2s; }
  .dots span.on { width: 24px; border-radius: 4px; background: var(--accent); }
  .form { padding: var(--space-4); display: flex; flex-direction: column; gap: var(--space-3); }
  .opt { display: flex; gap: var(--space-3); align-items: flex-start; justify-content: space-between; }
  .opt > span { display: flex; flex-direction: column; gap: 2px; }
  .opt > span > span { font-size: var(--fs-sm); }
  .warn { margin: 0; font-size: var(--fs-sm); display: flex; gap: 6px; align-items: flex-start; }
  .warn :global(svg) { flex: none; margin-top: 2px; }
  .sw { flex: none; position: relative; width: 48px; height: 28px; border-radius: var(--r-pill); background: var(--surface-2); border: 1px solid var(--border); transition: background 0.15s; }
  .knob { position: absolute; top: 3px; left: 3px; width: 20px; height: 20px; border-radius: 50%; background: var(--muted); transition: transform 0.15s, background 0.15s; }
  .sw[aria-checked="true"] { background: var(--accent); border-color: var(--accent); }
  .sw[aria-checked="true"] .knob { transform: translateX(20px); background: var(--on-accent); }
</style>
