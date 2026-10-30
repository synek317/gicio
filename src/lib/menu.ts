import { columnAt } from './lsp';
import { app } from './state.svelte';
import type { Side } from './types';

/** Right-click on code: remember where, the ContextMenu component renders the actions. */
export function openMenu(e: MouseEvent, path: string, side: Side, line: number, character?: number) {
  e.preventDefault();
  app.hover = null;
  app.menu = { x: e.clientX, y: e.clientY, path, side, line, character: character ?? columnAt(e.clientX, e.clientY) ?? 0 };
}
