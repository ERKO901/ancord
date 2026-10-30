<script lang="ts">
  import { hue, initials } from "../format";

  let { name, size = 46 }: { name: string; size?: number } = $props();
</script>

<span class="avatar" style="--h:{hue(name)};--sz:{size}px" aria-hidden="true">{initials(name)}</span>

<style>
  .avatar {
    flex: none;
    width: var(--sz); height: var(--sz);
    display: grid; place-items: center;
    border-radius: 50%;
    background: hsl(var(--h) var(--av-s) var(--av-l));
    color: var(--av-fg);
    font-weight: 600;
    font-size: calc(var(--sz) * 0.38);
    letter-spacing: 0.02em;
  }
</style>
