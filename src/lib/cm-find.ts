import { SearchQuery } from '@codemirror/search';
import { Prec, RangeSetBuilder, StateEffect, StateField, type Extension } from '@codemirror/state';
import { Decoration, EditorView, ViewPlugin, type DecorationSet, type ViewUpdate } from '@codemirror/view';
import type { FindResult, FindTarget } from './find.svelte';
import { editorFindKey } from './hotkeys';

/** The find bar driving a CodeMirror editor: the editor only renders the lines on screen, so it is
 *  searched through its document rather than its DOM. */

const MAX_MATCHES = 10000;

const setQuery = StateEffect.define<SearchQuery | null>();
const queryField = StateField.define<SearchQuery | null>({
  create: () => null,
  update: (q, tr) => tr.effects.reduce((acc, e) => (e.is(setQuery) ? e.value : acc), q),
});

const matchMark = Decoration.mark({ class: 'cm-find-match' });
const currentMark = Decoration.mark({ class: 'cm-find-match cm-find-current' });

/** Marks the matches on screen; the one that is selected is the current one. */
const highlighter = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;
    constructor(view: EditorView) {
      this.decorations = this.build(view);
    }
    update(u: ViewUpdate) {
      if (u.docChanged || u.viewportChanged || u.selectionSet || u.startState.field(queryField) !== u.state.field(queryField)) this.decorations = this.build(u.view);
    }
    build(view: EditorView): DecorationSet {
      const q = view.state.field(queryField);
      if (!q?.valid) return Decoration.none;
      const b = new RangeSetBuilder<Decoration>();
      const sel = view.state.selection.main;
      for (const { from, to } of view.visibleRanges) {
        const cur = q.getCursor(view.state, from, to);
        for (let m = cur.next(); !m.done; m = cur.next()) {
          const { from: a, to: z } = m.value;
          if (a < z) b.add(a, z, a === sel.from && z === sel.to ? currentMark : matchMark);
        }
      }
      return b.finish();
    }
  },
  { decorations: (p) => p.decorations },
);

/** Find shortcuts open the app's find bar (or project search) instead of the editor's own panel. */
const findKeys = Prec.highest(EditorView.domEventHandlers({ keydown: (e) => editorFindKey(e) }));

export const findExtensions: Extension = [queryField, highlighter, findKeys];

export function cmFindTarget(view: EditorView): FindTarget {
  let report: (r: FindResult) => void = () => {};
  let query: SearchQuery | null = null;
  let all: { from: number; to: number }[] = [];

  function count() {
    all = [];
    if (!query?.valid) return;
    const cur = query.getCursor(view.state);
    for (let m = cur.next(); !m.done && all.length < MAX_MATCHES; m = cur.next()) if (m.value.from < m.value.to) all.push(m.value);
  }
  const currentIndex = () => {
    const s = view.state.selection.main;
    return all.findIndex((m) => m.from === s.from && m.to === s.to);
  };
  const tell = () => report({ count: all.length, current: currentIndex() });
  function select(i: number) {
    const m = all[i];
    view.dispatch({ selection: { anchor: m.from, head: m.to }, effects: EditorView.scrollIntoView(m.from, { y: 'center' }) });
    report({ count: all.length, current: i });
  }

  view.dispatch({
    effects: StateEffect.appendConfig.of(
      EditorView.updateListener.of((u) => {
        if (!query) return;
        if (u.docChanged) count();
        if (u.docChanged || u.selectionSet) tell();
      }),
    ),
  });

  return {
    show(spec, r) {
      report = r;
      query = spec ? new SearchQuery({ search: spec.query, caseSensitive: spec.caseSensitive, regexp: spec.regex, wholeWord: spec.wholeWord, literal: true }) : null;
      view.dispatch({ effects: setQuery.of(query) });
      count();
      if (!all.length) return tell();
      // Searching as you type keeps the match at the cursor, else takes the next one.
      const from = view.state.selection.main.from;
      const i = all.findIndex((m) => m.from >= from);
      select(i < 0 ? 0 : i);
    },
    step(dir) {
      if (!all.length) return;
      const from = view.state.selection.main.from;
      const i = dir > 0 ? all.findIndex((m) => m.from > from) : all.findLastIndex((m) => m.from < from);
      select(i >= 0 ? i : dir > 0 ? 0 : all.length - 1);
    },
    selectedText() {
      const s = view.state.selection.main;
      // A whole selected line is where a jump landed, not something to look for.
      const line = view.state.doc.lineAt(s.from);
      return s.from === line.from && s.to === line.to ? '' : view.state.sliceDoc(s.from, s.to).trim();
    },
    focus: () => view.focus(),
  };
}
