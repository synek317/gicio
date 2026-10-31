import { editFile, revealLine } from './actions';
import { go } from './nav';
import { findPathRefs, PathRef } from './path-ref';
import { app } from './state.svelte';

/** Repo-relative path a mention points at, or null when it isn't a file of this repository.
 *  Partial paths (`lsp/client.rs`) map to the one review file ending with them; bare names
 *  (`lsp.ts`) count only then, since otherwise they are too often just identifiers. */
function resolve(ref: PathRef): string | null {
  const repo = app.snap?.repo;
  if (ref.path.startsWith('/')) return repo && ref.path.startsWith(`${repo}/`) ? ref.path.slice(repo.length + 1) : null;
  if (app.files.has(ref.path)) return ref.path;
  const same = [...app.files.keys()].filter((p) => p.endsWith(`/${ref.path}`));
  if (same.length === 1) return same[0];
  return ref.isBareName ? null : ref.path;
}

const attr = (s: string) => s.replace(/&/g, '&amp;').replace(/"/g, '&quot;');

function open(path: string, line: number | null, label: string): string {
  return `<a class="path-link" href="#" data-path="${attr(path)}"${line ? ` data-line="${line}"` : ''} title="Open ${attr(path)}${line ? `:${line}` : ''}">${label}`;
}

/** Wraps file mentions in already-escaped text in links. Paths contain no characters that
 *  escaping changes, so matching on the escaped form is safe. */
export function linkPathsInHtml(escaped: string): string {
  let out = '';
  let at = 0;
  for (const m of findPathRefs(escaped)) {
    const path = resolve(m.ref);
    if (!path) continue;
    out += escaped.slice(at, m.index) + open(path, m.ref.line, m.text) + '</a>';
    at = m.index + m.text.length;
  }
  return out + escaped.slice(at);
}

/** Links a whole inline code span when it is exactly one file mention. */
export function linkCodeSpan(code: string, html: string): string {
  const ref = PathRef.parse(code);
  const path = ref && resolve(ref);
  return ref && path ? open(path, ref.line, html) + '</a>' : html;
}

const lineInDiff = (path: string, line: number) => app.files.get(path)?.hunks.some((h) => h.lines.some((l) => l.newNo === line)) ?? false;

/** Shows a mentioned file: the diff where it has that line, otherwise the file itself. */
function show(path: string, line: number | null) {
  if (line != null && lineInDiff(path, line)) revealLine(path, 'new', line);
  else if (line == null && app.files.has(path)) go({ kind: 'file', path });
  else if (app.files.get(path)?.status === 'deleted') go({ kind: 'file', path });
  else editFile(path, line);
}

/** One listener for every rendered AI text; returns the cleanup. */
export function handlePathLinks(): () => void {
  const onClick = (e: MouseEvent) => {
    const a = (e.target as Element | null)?.closest?.<HTMLElement>('a.path-link');
    if (!a?.dataset.path) return;
    e.preventDefault();
    e.stopPropagation();
    show(a.dataset.path, a.dataset.line ? Number(a.dataset.line) : null);
  };
  document.addEventListener('click', onClick, true);
  return () => document.removeEventListener('click', onClick, true);
}
