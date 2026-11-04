export type PanelSide = 'left' | 'right';

export interface PanelPref {
  width: number;
  visible: boolean;
}

export type PanelPrefs = Record<PanelSide, PanelPref>;

export const DEFAULT_PANEL_WIDTH: Record<PanelSide, number> = { left: 290, right: 400 };

export const defaultPanelPrefs = (): PanelPrefs => ({
  left: { width: DEFAULT_PANEL_WIDTH.left, visible: true },
  right: { width: DEFAULT_PANEL_WIDTH.right, visible: true },
});
