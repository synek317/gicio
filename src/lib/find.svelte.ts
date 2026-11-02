import { domFindTarget } from './find-dom';
import { compileFind, type FindOptions, type FindSpec } from './find-query';

export interface FindResult {
  count: number;
  /** Index of the current match, -1 for none. */
  current: number;
}

/** Where the find bar searches: the diff on screen, or an open editor. */
export interface FindTarget {
  /** Marks the matches of `spec` (nothing for null). `report` hears the result now and again
   *  whenever the content changes. */
  show(spec: FindSpec | null, report: (r: FindResult) => void): void;
  /** Moves to the next (1) or previous (-1) match. */
  step(dir: 1 | -1): void;
  /** One-line text selected in the target, to start a search with. */
  selectedText(): string;
  /** Takes the keyboard back when the find bar closes. */
  focus?(): void;
}

/** The find bar (⌘F) over the main view. */
class FindState {
  open = $state(false);
  query = $state('');
  opts = $state<FindOptions>({ caseSensitive: false, wholeWord: false, regex: false });
  count = $state(0);
  current = $state(-1);
  error = $state('');
  /** Bumped to put the cursor in the find box. */
  focus = $state(0);
}

export const find = new FindState();

let target: FindTarget = domFindTarget;

function report(r: FindResult) {
  find.count = r.count;
  find.current = r.current;
}

function spec(): FindSpec | null {
  if (!find.open || !find.query) return null;
  try {
    compileFind({ query: find.query, ...find.opts });
    find.error = '';
    return { query: find.query, ...find.opts };
  } catch (e) {
    find.error = e instanceof Error ? e.message : String(e);
    return null;
  }
}

/** Searches again after the query, an option or the target changed. */
export function refreshFind() {
  const s = spec();
  if (!s) report({ count: 0, current: -1 });
  target.show(s, report);
}

/** An editor takes over the find bar while it is open; null gives it back to the diff. */
export function setFindTarget(t: FindTarget | null) {
  target.show(null, report);
  target = t ?? domFindTarget;
  refreshFind();
}

/** Selected text worth searching for: one line, not too long. */
export function seedText(): string {
  const t = document.activeElement?.closest('.find-bar, .search-panel') ? '' : target.selectedText();
  return t && !t.includes('\n') && t.length <= 200 ? t : '';
}

export function openFind() {
  const seed = seedText();
  if (seed) find.query = seed;
  // The bar searches as soon as it shows (and whenever its query changes).
  find.open = true;
  find.focus++;
}

export function closeFind() {
  find.open = false;
  refreshFind();
  target.focus?.();
}

export function findStep(dir: 1 | -1) {
  if (!find.open) return openFind();
  if (find.count) target.step(dir);
}

/** Starts the find bar with a query from elsewhere (a project search result), without opening it. */
export function presetFind(query: string, opts: FindOptions) {
  find.query = query;
  find.opts = { ...opts };
}
