<script lang="ts">
  import type { PanelSide } from '../panel-prefs';
  import { panelWidth, resetPanelWidth, resizePanel, savePanels } from '../panels.svelte';

  const KEY_STEP_PX = 16;

  let { side }: { side: PanelSide } = $props();
  /** Where the drag started; null when not dragging. */
  let start: { x: number; width: number } | null = null;
  let dragging = $state(false);

  /** Moving the pointer away from the panel's outer edge makes it wider. */
  const widthFor = (x: number, s: { x: number; width: number }) => s.width + (side === 'left' ? x - s.x : s.x - x);

  function down(e: PointerEvent) {
    if (e.button !== 0) return;
    e.currentTarget instanceof Element && e.currentTarget.setPointerCapture(e.pointerId);
    start = { x: e.clientX, width: panelWidth(side) };
    dragging = true;
  }

  function move(e: PointerEvent) {
    if (start) resizePanel(side, widthFor(e.clientX, start));
  }

  function up() {
    if (!start) return;
    start = null;
    dragging = false;
    savePanels();
  }

  function key(e: KeyboardEvent) {
    if (e.key !== 'ArrowLeft' && e.key !== 'ArrowRight') return;
    e.preventDefault();
    e.stopPropagation();
    const towardsRight = e.key === 'ArrowRight';
    const wider = side === 'left' ? towardsRight : !towardsRight;
    resizePanel(side, panelWidth(side) + (wider ? KEY_STEP_PX : -KEY_STEP_PX));
    savePanels();
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div
  class="panel-resizer {side}"
  class:dragging
  role="separator"
  aria-orientation="vertical"
  aria-label="Resize the {side} panel"
  aria-valuenow={panelWidth(side)}
  tabindex="0"
  title="Drag to resize, double-click for the default width"
  onmousedown={(e) => e.preventDefault()}
  onpointerdown={down}
  onpointermove={move}
  onpointerup={up}
  onpointercancel={up}
  ondblclick={() => resetPanelWidth(side)}
  onkeydown={key}
></div>
