import { settings } from './app-settings.svelte';
import { DEFAULT_PANEL_WIDTH, type PanelPref, type PanelSide } from './panel-prefs';
import { app } from './state.svelte';

const MIN_WIDTH: Record<PanelSide, number> = { left: 180, right: 260 };
/** Largest share of the window one panel may take, so the diff always stays usable. */
const MAX_WINDOW_PERCENT = 40;

function clamp(side: PanelSide, width: number): number {
  const max = Math.max(MIN_WIDTH[side], (window.innerWidth * MAX_WINDOW_PERCENT) / 100);
  return Math.round(Math.min(max, Math.max(MIN_WIDTH[side], width)));
}

export const panelPref = (side: PanelSide): PanelPref => settings.value.prefs.panels[side];

/** The width on screen, which a window smaller than when it was set can limit. */
export const panelWidth = (side: PanelSide): number => clamp(side, panelPref(side).width);

/** The grid column of a panel. */
export function panelColumn(side: PanelSide): string {
  const { visible, width } = panelPref(side);
  return visible ? `min(${width}px, ${MAX_WINDOW_PERCENT}vw)` : '0px';
}

export function savePanels() {
  settings.save($state.snapshot(settings.value)).catch((e) => app.toast(String(e), 'error'));
}

/** Resizes without saving, as dragging does this many times a second. */
export function resizePanel(side: PanelSide, width: number) {
  panelPref(side).width = clamp(side, width);
}

export function resetPanelWidth(side: PanelSide) {
  resizePanel(side, DEFAULT_PANEL_WIDTH[side]);
  savePanels();
}

export function togglePanel(side: PanelSide) {
  panelPref(side).visible = !panelPref(side).visible;
  savePanels();
}

/** Brings back a hidden panel when something is about to happen in it. */
export function showPanel(side: PanelSide) {
  if (!panelPref(side).visible) togglePanel(side);
}
