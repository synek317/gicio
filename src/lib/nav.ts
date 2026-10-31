import { api } from './api';
import { app, type View } from './state.svelte';

const scroller = () => document.getElementById('diff-scroll');

function show(view: View, scroll = 0) {
  app.view = view;
  if (view.kind === 'chunk') api.setLastChunk(view.id).catch(() => {});
  app.selection = [];
  app.selAnchor = null;
  app.focusedHunk = app.viewHunks[0]?.id ?? null;
  app.composer = null;
  app.menu = null;
  requestAnimationFrame(() => scroller()?.scrollTo({ top: scroll }));
}

function leaveEditor(): boolean {
  if (app.view?.kind === 'source' && app.editorDirty && !window.confirm('Discard unsaved edits?')) return false;
  app.editorDirty = false;
  return true;
}

/** Navigates to a view; with `push` the current one is remembered for Back. */
export function go(view: View, push = true) {
  if (!leaveEditor()) return;
  if (push && app.view) {
    app.history = [...app.history.slice(-99), { view: app.view, scroll: scroller()?.scrollTop ?? 0 }];
    app.future = [];
  }
  show(view);
}

export function back() {
  const prev = app.history.at(-1);
  if (!prev || !leaveEditor()) return;
  app.history = app.history.slice(0, -1);
  if (app.view) app.future = [{ view: app.view, scroll: scroller()?.scrollTop ?? 0 }, ...app.future];
  show(prev.view, prev.scroll);
}

export function forward() {
  const next = app.future[0];
  if (!next || !leaveEditor()) return;
  app.future = app.future.slice(1);
  if (app.view) app.history = [...app.history, { view: app.view, scroll: scroller()?.scrollTop ?? 0 }];
  show(next.view, next.scroll);
}

/** Remembers the current place for Back without leaving the view (e.g. before an in-place jump). */
export function remember() {
  if (!app.view) return;
  app.history = [...app.history.slice(-99), { view: app.view, scroll: scroller()?.scrollTop ?? 0 }];
  app.future = [];
}
