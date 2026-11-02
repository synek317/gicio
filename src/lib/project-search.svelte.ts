import { api } from './api';
import type { FindOptions } from './find-query';
import { presetFind, seedText } from './find.svelte';
import { openLocation } from './lsp';
import { app } from './state.svelte';
import type { FileHits, LineHit, SearchQuery, SearchResults } from './types';

/** The project search in the sidebar (⇧⌘F). */
class ProjectSearch {
  query = $state('');
  opts = $state<FindOptions>({ caseSensitive: false, wholeWord: false, regex: false });
  /** Comma-separated globs, as in VS Code ("*.rs, src/lib"). */
  include = $state('');
  exclude = $state('');
  showFilters = $state(false);
  results = $state<SearchResults | null>(null);
  busy = $state(false);
  error = $state('');
  /** Bumped to put the cursor in the search box. */
  focus = $state(0);
  /** Result files folded away, by path. */
  collapsed = $state<Record<string, boolean>>({});
  /** The result opened last, `${path}:${line}`. */
  active = $state<string | null>(null);
}

export const search = new ProjectSearch();

let seq = 0;
let timer: ReturnType<typeof setTimeout> | undefined;
let lastKey = '';

function currentQuery(): SearchQuery {
  return { pattern: search.query, ...search.opts, include: search.include, exclude: search.exclude };
}

/** Searches after a pause in typing; the same query again only with `force` (refresh). */
export function runSearch(delay = 250, force = false) {
  const q = currentQuery();
  const key = JSON.stringify(q);
  if (key === lastKey && !force) return;
  lastKey = key;
  clearTimeout(timer);
  const id = ++seq;
  if (!q.pattern) {
    search.results = null;
    search.error = '';
    search.busy = false;
    return;
  }
  timer = setTimeout(async () => {
    search.busy = true;
    try {
      const r = await api.searchProject(q);
      if (id !== seq) return;
      search.results = r;
      search.error = '';
      search.collapsed = {};
    } catch (e) {
      if (id !== seq) return;
      search.results = null;
      search.error = String(e);
    } finally {
      if (id === seq) search.busy = false;
    }
  }, delay);
}

export function openProjectSearch() {
  const seed = seedText();
  if (seed) search.query = seed;
  app.sidebarTab = 'search';
  search.focus++;
}

/** Shows a result: in the diff when it is on screen there, otherwise in the file, with the match
 *  selected; F3 / ⌘G then go on through the file. */
export function openHit(file: FileHits, hit: LineHit) {
  search.active = `${file.path}:${hit.line}`;
  presetFind(search.query, search.opts);
  openLocation({ absPath: file.absPath, repoPath: file.path, line: hit.line - 1, character: hit.column, preview: null }, hit.length);
}
