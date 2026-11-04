<script lang="ts">
  import type { PanelSide } from '../panel-prefs';
  import { panelPref, togglePanel } from '../panels.svelte';

  let { side }: { side: PanelSide } = $props();
  const visible = $derived(panelPref(side).visible);
  const title = $derived(`${visible ? 'Hide' : 'Show'} the ${side} panel (${side === 'left' ? 'b' : '⇧B'})`);
</script>

<button class="ghost panel-toggle" class:active={visible} {title} aria-label={title} aria-pressed={visible} onclick={() => togglePanel(side)}>
  <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
    <rect x="1.5" y="2.5" width="13" height="11" rx="1.5" fill="none" stroke="currentColor" />
    <rect x={side === 'left' ? 2 : 10} y="3" width="4" height="10" fill="currentColor" opacity={visible ? 1 : 0.25} />
  </svg>
</button>
