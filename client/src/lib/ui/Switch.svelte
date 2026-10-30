<script lang="ts">
  let { checked, onchange, label }: { checked: boolean; onchange: (v: boolean) => void; label: string } = $props();
</script>

<button type="button" class="sw" role="switch" aria-checked={checked} aria-label={label} onclick={() => onchange(!checked)}>
  <span class="knob"></span>
</button>

<style>
  .sw {
    flex: none; position: relative; width: 48px; height: 28px; border-radius: var(--r-pill);
    background: var(--surface-2); border: 1px solid var(--border); transition: background 0.15s;
  }
  .knob {
    position: absolute; top: 3px; left: 3px; width: 20px; height: 20px; border-radius: 50%;
    background: var(--muted); transition: transform 0.15s, background 0.15s;
  }
  .sw[aria-checked="true"] { background: var(--accent); border-color: var(--accent); }
  .sw[aria-checked="true"] .knob { transform: translateX(20px); background: var(--on-accent); }
</style>
