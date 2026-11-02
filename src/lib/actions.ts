import { api } from './api';
import { confirm } from './confirm';
import { settings } from './app-settings.svelte';
import { changedKeys, parseKey } from './keys';
import { hunkProgress } from './progress';
import { go } from './nav';
import { scrollToLine } from './scroll';
import { afterAccept } from './blank-lines';
import { previewOff } from './preview';
export { scrollToLine } from './scroll';
import { app } from './state.svelte';
import type { AnchorInput, Hunk, MarkStatus, SendMode, Snapshot } from './types';

export async function run(p: Promise<Snapshot>): Promise<boolean> {
  try {
    app.applySnapshot(await p);
    return true;
  } catch (e) {
    app.toast(String(e), 'error');
    return false;
  }
}

export { back, forward, go } from './nav';

function targetKeys(): string[] {
  if (app.selection.length) return app.selection;
  const h = app.focusedHunk ? app.viewHunkMap.get(app.focusedHunk) : undefined;
  return h ? changedKeys(h) : [];
}

export async function markKeys(keys: string[], status: MarkStatus | null, advance = true) {
  if (!keys.length) return;
  if (await run(api.setMarks(keys.map(parseKey), status))) {
    if (advance) app.selection = [];
    if (status && advance) focusNextPending();
    if (status === 'accepted') afterAccept(new Set(keys.map((k) => parseKey(k).path)), (rest) => markKeys(rest, 'accepted', false));
  }
  previewOff();
}

export const markTarget = (status: MarkStatus | null) => markKeys(targetKeys(), status);

export async function markView(status: MarkStatus) {
  const keys = app.orderedKeys.filter((k) => !app.marks.has(k));
  await markKeys(keys, status);
}

export async function acceptRestAndNext() {
  await markView('accepted');
  if (settings.value.prefs.autoAdvance) nextPendingChunk();
}

export function focusHunk(id: string | null) {
  app.focusedHunk = id;
  if (id) document.getElementById(`hunk-${id}`)?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
}

export function moveFocus(delta: number) {
  const hs = app.viewHunks;
  if (!hs.length) return;
  const i = hs.findIndex((h) => h.id === app.focusedHunk);
  const next = Math.min(hs.length - 1, Math.max(0, i < 0 ? 0 : i + delta));
  focusHunk(hs[next].id);
}

function focusNextPending() {
  const hs = app.viewHunks;
  const cur = hs.findIndex((h) => h.id === app.focusedHunk);
  if (cur >= 0 && hunkProgress(hs[cur], app.marks).pending > 0) return;
  const order = [...hs.slice(cur + 1), ...hs.slice(0, Math.max(cur, 0))];
  const next = order.find((h) => hunkProgress(h, app.marks).pending > 0);
  if (next) focusHunk(next.id);
}

export function moveChunk(delta: number) {
  const cs = app.snap?.chunks ?? [];
  const v = app.view;
  const i = v?.kind === 'chunk' ? cs.findIndex((c) => c.id === v.id) : -1;
  const next = cs[Math.min(cs.length - 1, Math.max(0, i + delta))];
  if (next) go({ kind: 'chunk', id: next.id });
}

export function nextPendingChunk() {
  const cs = app.snap?.chunks ?? [];
  const v = app.view;
  const i = v?.kind === 'chunk' ? cs.findIndex((c) => c.id === v.id) : -1;
  const order = [...cs.slice(i + 1), ...cs.slice(0, Math.max(i, 0))];
  const next = order.find((c) => app.chunkProgress(c).pending > 0);
  if (next) go({ kind: 'chunk', id: next.id });
  else if (app.hasLater) {
    app.toast('All steps done. Showing what you left for later.');
    go({ kind: 'later' });
  } else app.toast('Everything is reviewed 🎉');
}

/** Selection by gutter: plain click starts, shift extends, cmd toggles. */
export function selectLine(key: string, mode: 'set' | 'extend' | 'toggle') {
  const order = app.orderedKeys;
  app.selHead = key;
  if (mode === 'toggle') {
    app.selection = app.selSet.has(key) ? app.selection.filter((k) => k !== key) : [...app.selection, key];
    app.selAnchor = key;
    return;
  }
  if (mode === 'extend' && app.selAnchor) {
    const a = order.indexOf(app.selAnchor);
    const b = order.indexOf(key);
    if (a >= 0 && b >= 0) {
      app.selection = order.slice(Math.min(a, b), Math.max(a, b) + 1);
      return;
    }
  }
  app.selection = [key];
  app.selAnchor = key;
}

function currentAnchor(): AnchorInput | null {
  if (app.selection.length) {
    const keys = app.selection.map(parseKey);
    const first = keys[0];
    const same = keys.filter((k) => k.path === first.path && k.side === first.side).map((k) => k.line);
    return { path: first.path, side: first.side, start: Math.min(...same), end: Math.max(...same) };
  }
  const h = app.focusedHunk ? app.viewHunkMap.get(app.focusedHunk) : undefined;
  if (!h) return null;
  if (h.newLen > 0) return { path: h.path, side: 'new', start: h.newStart, end: h.newStart + h.newLen - 1 };
  return { path: h.path, side: 'old', start: h.oldStart, end: h.oldStart + Math.max(h.oldLen, 1) - 1 };
}

/** The step on screen, which new conversations belong to. */
const stepOnScreen = () => (app.view?.kind === 'chunk' ? app.view.id : null);

/** A new conversation about the selected lines (in the box next to them), the focused hunk,
 *  else the current step, else the whole change. */
export function openComposer(anchored = true) {
  if (anchored && app.selection.length) {
    app.selectionFocus++;
    return;
  }
  const anchor = anchored ? currentAnchor() : null;
  const chunk = stepOnScreen();
  if (!anchor && !chunk) return focusChat();
  app.rightTab = 'conversations';
  app.convScope = 'here';
  app.composer = { anchor, chunk };
}

/** Opens the conversation about the whole change. */
export function focusChat() {
  app.rightTab = 'conversations';
  app.convScope = 'whole';
  app.composer = null;
  app.chatFocus++;
}

/** Shows the history a conversation belongs to: the whole change, or what is on screen. */
export function showConversation(t: { id: string; chunk: string | null; anchor: AnchorInput | null }) {
  app.activeThread = t.id;
  app.rightTab = 'conversations';
  app.convScope = !t.chunk && !t.anchor ? 'whole' : 'here';
}

const LATER_TOAST = 'Left for later. It waits in the last step, “Resolve issues left for later”.';

/** Starts a conversation about `anchor` (lines) in `chunk` (a step); both null: the whole change. */
export async function startConversation(chunk: string | null, anchor: AnchorInput | null, mode: SendMode, body: string, allowEdits: boolean) {
  if (!body.trim()) return;
  try {
    const id = await api.createThread(chunk, anchor, allowEdits);
    showConversation({ id, chunk, anchor });
    if ((await run(api.sendMessage(id, body, mode, null))) && mode === 'later') app.toast(LATER_TOAST);
  } catch (e) {
    app.toast(String(e), 'error');
  }
}

/** Sends what was typed in the composer of the conversations panel. */
export async function startThread(mode: SendMode, body: string, allowEdits: boolean) {
  const c = app.composer;
  if (!c) return;
  app.composer = null;
  await startConversation(c.chunk, c.anchor, mode, body, allowEdits);
}

/** Sends what was typed in the box next to the selected lines. */
export async function askAboutSelection(mode: SendMode, body: string, allowEdits: boolean) {
  const anchor = currentAnchor();
  if (!anchor) return;
  app.selection = [];
  await startConversation(stepOnScreen(), anchor, mode, body, allowEdits);
}

export async function plan(replan: boolean) {
  if (await run(replan ? api.replanAi() : api.planAi())) {
    app.toast(`AI plan ready: ${app.snap?.chunks.length} chunks`);
    const first = app.snap?.chunks.find((c) => app.chunkProgress(c).pending > 0) ?? app.snap?.chunks[0];
    if (first && !replan) go({ kind: 'chunk', id: first.id });
  }
}


export function revealThread(t: { id: string; chunk: string | null; anchor: AnchorInput | null }) {
  showConversation(t);
  const a = t.anchor;
  if (t.chunk && app.chunk(t.chunk) && !(app.view?.kind === 'chunk' && app.view.id === t.chunk)) go({ kind: 'chunk', id: t.chunk });
  if (a && !(a as { outdated?: boolean }).outdated) setTimeout(() => scrollToLine(a.path, a.side, a.start), 60);
}

/** Throws away all progress for this review and starts again. */
export async function startOver() {
  const msg = 'Start this review over? All accepted/later marks, the queue and discussions are discarded. Code is not touched.';
  if (settings.value.prefs.confirmStartOver && !(await confirm(msg, 'Start over'))) return;
  if (!(await run(api.resetReview()))) return;
  app.history = [];
  app.future = [];
  const first = app.snap?.chunks[0];
  if (first) go({ kind: 'chunk', id: first.id }, false);
  if (settings.value.start.order === 'ai') plan(false);
}

export function currentChunkId(): string | null {
  const v = app.view;
  if (v?.kind === 'chunk') return v.id;
  for (const h of app.history.toReversed()) if (h.view.kind === 'chunk') return h.view.id;
  return null;
}

/** Opens a repo file in the main window (editable, with the language server). */
export function editFile(path: string, line: number | null = null) {
  go({ kind: 'source', title: path, absPath: `${app.snap!.repo}/${path}`, repoPath: path, line });
}

/** Shows a diff line wherever it is: the step containing it, else the file view of the diff. */
export function revealLine(path: string, side: 'old' | 'new', line: number) {
  const holds = (h: { path: string; lines: { oldNo: number | null; newNo: number | null }[] }) =>
    h.path === path && h.lines.some((l) => (side === 'old' ? l.oldNo : l.newNo) === line);
  const chunk = app.snap?.chunks.find((c) => app.chunkHunks(c).some(holds));
  const here = app.viewHunks.some(holds);
  if (!here) go(chunk ? { kind: 'chunk', id: chunk.id } : { kind: 'file', path });
  setTimeout(() => scrollToLine(path, side, line), here ? 0 : 120);
}

/** Lines the selection (or focused hunk) covers, for anchoring comments or an AI review. */
export function selectionAnchor(): AnchorInput | null {
  return currentAnchor();
}

/** Closes the in-place editor; unsaved changes are discarded only after asking. */
export async function closeEditor(): Promise<boolean> {
  if (app.editing && app.editorDirty && !(await confirm('Discard changes?'))) return false;
  app.editorDirty = false;
  app.editing = null;
  return true;
}

/** Where the cursor goes when editing a hunk: the line under the pointer, else its first added line. */
function editTarget(h: Hunk): number | null {
  const news = h.lines.filter((l) => l.newNo != null);
  const p = app.pointerCode;
  if (p && p.path === h.path && news.some((l) => l.newNo === p.line)) return p.line;
  return (news.find((l) => l.kind === 'added') ?? news[0])?.newNo ?? null;
}

/** Edits a hunk right in the diff, with the cursor on `line` (default: see `editTarget`). */
export async function editHunk(id: string, line: number | null = null) {
  const h = app.viewHunkMap.get(id);
  if (!h || app.files.get(h.path)?.status === 'deleted') return;
  if (app.editing?.hunk !== id && !(await closeEditor())) return;
  const target = line ?? editTarget(h);
  const row = target == null ? null : document.querySelector(`#hunk-${CSS.escape(id)} [data-line="${CSS.escape(`${h.path}|new|${target}`)}"]`);
  const top = document.getElementById('diff-scroll')?.getBoundingClientRect().top ?? 0;
  app.focusedHunk = id;
  app.editing = { hunk: id, line: target, offset: row ? row.getBoundingClientRect().top - top : null };
}

export function editFocused() {
  if (app.view?.kind === 'source') return;
  const h = app.focusedHunk ? app.viewHunkMap.get(app.focusedHunk) : app.viewHunks[0];
  if (h) editHunk(h.id);
}

/** Edits the diff line in place when it is on screen, else opens the file at it. */
export function editLine(path: string, line: number) {
  const h = app.viewHunks.find((h) => h.path === path && h.lines.some((l) => l.newNo === line));
  if (h) editHunk(h.id, line);
  else editFile(path, line);
}
