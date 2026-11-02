import { go, remember } from './nav';
import { scrollToLine } from './scroll';
import { api } from './api';
import { app } from './state.svelte';
import { settings } from './app-settings.svelte';
import type { SourceLocation } from './types';

let timer: ReturnType<typeof setTimeout> | undefined;
let hideTimer: ReturnType<typeof setTimeout> | undefined;

/** How the UI names the project's language server, e.g. "rust-analyzer". */
export const lspName = () => app.lsp.server ?? 'language server';

/** Whether the project's language server (running or not) understands this file. */
export const lspHandles = (path: string) => app.lsp.extensions.includes(path.split('.').pop()?.toLowerCase() ?? '');

/** Whether to show the prompt about a missing (or, when asked for, failed) language server. */
export function lspPromptShown(): boolean {
  if (app.lspPrompt === 'open') return app.lsp.state === 'missing' || app.lsp.state === 'failed';
  return app.lspPrompt === 'auto' && app.lsp.state === 'missing' && settings.value.prefs.offerLspInstall;
}

export const lspUsable = (path: string) => lspHandles(path) && (app.lsp.state === 'ready' || app.lsp.state === 'indexing');

/** UTF-16 column under the pointer inside a rendered code line (tokens carry `data-col`). */
export function columnAt(x: number, y: number): number | null {
  const doc = document as Document & { caretRangeFromPoint?: (x: number, y: number) => Range | null };
  const r = doc.caretRangeFromPoint?.(x, y);
  const el = r?.startContainer.parentElement?.closest<HTMLElement>('[data-col]');
  if (!r || !el) return null;
  return Number(el.dataset.col) + r.startOffset;
}

export function hoverCode(e: MouseEvent, path: string, line: number) {
  if (!lspUsable(path)) return;
  const character = columnAt(e.clientX, e.clientY);
  app.pointerCode = character == null ? null : { path, line, character };
  clearTimeout(timer);
  const { clientX: x, clientY: y } = e;
  timer = setTimeout(async () => {
    const col = columnAt(x, y);
    if (col == null) return;
    try {
      const text = await api.lspHover(path, line - 1, col);
      clearTimeout(hideTimer);
      app.hover = text ? { x, y, text } : null;
    } catch {
      app.hover = null;
    }
  }, settings.value.prefs.hoverDelayMs);
}

export function leaveCode() {
  clearTimeout(timer);
  hideTimer = setTimeout(() => (app.hover = null), 250);
}

export function keepHover() {
  clearTimeout(hideTimer);
}

/** Shows a location: scrolls when it is already on screen in the diff, otherwise opens the file. */
export function openLocation(loc: SourceLocation) {
  const v = app.view;
  const inDiff = v && (v.kind === 'chunk' || v.kind === 'file' || v.kind === 'later');
  if (inDiff && loc.repoPath && document.querySelector(`[data-line="${CSS.escape(`${loc.repoPath}|new|${loc.line + 1}`)}"]`)) {
    remember();
    scrollToLine(loc.repoPath, 'new', loc.line + 1);
    return;
  }
  go({ kind: 'source', title: loc.repoPath ?? loc.absPath, absPath: loc.absPath, repoPath: loc.repoPath, line: loc.line + 1 });
}

/** Definition in the main window; several candidates are listed like references. */
export async function gotoDefinition(path: string, line: number, col: number) {
  app.hover = null;
  try {
    const locs = await api.lspDefinition(path, line - 1, col);
    if (!locs.length) app.toast('No definition found');
    else if (locs.length === 1) openLocation(locs[0]);
    else go({ kind: 'refs', title: 'Definitions', locs });
  } catch (err) {
    app.toast(String(err), 'error');
  }
}

export async function findReferences(path: string, line: number, col: number) {
  app.hover = null;
  if (app.lsp.state === 'indexing') app.toast(`Waiting for ${lspName()} to load the whole project…`);
  try {
    const locs = await api.lspReferences(path, line - 1, col);
    if (!locs.length) return app.toast('No references found');
    const symbol = locs.find((l) => l.repoPath === path && l.line === line - 1)?.preview ?? `${path}:${line}`;
    go({ kind: 'refs', title: `References: ${symbol}`, locs });
  } catch (err) {
    app.toast(String(err), 'error');
  }
}

export function definitionAtPointer(e: MouseEvent, path: string, line: number) {
  if (!lspUsable(path)) return;
  const col = columnAt(e.clientX, e.clientY);
  if (col != null) gotoDefinition(path, line, col);
}

export function pollLspStatus(): () => void {
  const tick = async () => {
    try {
      app.lsp = await api.lspStatus();
    } catch {
      /* no session yet */
    }
  };
  tick();
  const id = setInterval(tick, 2000);
  return () => clearInterval(id);
}
