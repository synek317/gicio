import { api } from './api';
import { isCSharpDecl } from './items-csharp';
import { lspUsable } from './lsp';
import type { DiffLine, Hunk } from './types';

export interface Extra { before: number; after: number }

/** First/last old and new line numbers a hunk shows. */
function edges(h: Hunk) {
  const olds = h.lines.map((l) => l.oldNo).filter((n): n is number => n != null);
  const news = h.lines.map((l) => l.newNo).filter((n): n is number => n != null);
  return { topOld: olds[0], topNew: news[0], botOld: olds.at(-1), botNew: news.at(-1) };
}

/** Room for extra context: up to the neighbouring hunks of the same file and the file's ends. */
export function room(h: Hunk, fileHunks: Hunk[], lineCount: number): Extra {
  const { topNew, botNew } = edges(h);
  if (topNew == null || botNew == null) return { before: 0, after: 0 };
  const others = fileHunks.filter((o) => o.id !== (h.baseId ?? h.id));
  const prevEnd = Math.max(0, ...others.filter((o) => o.newStart + o.newLen - 1 < topNew).map((o) => o.newStart + o.newLen - 1));
  const nextStart = Math.min(lineCount + 1, ...others.filter((o) => o.newStart > botNew).map((o) => o.newStart));
  return { before: Math.max(0, topNew - 1 - prevEnd), after: Math.max(0, nextStart - 1 - botNew) };
}

/** The hunk with `extra` unchanged lines of the new file added above and below. */
export function extendHunk(h: Hunk, extra: Extra, newLines: string[]): Hunk {
  if (!extra.before && !extra.after) return h;
  const { topOld, topNew, botOld, botNew } = edges(h);
  if (topNew == null || botNew == null) return h;
  const ctx = (n: number, delta: number | null): DiffLine => ({ kind: 'context', newNo: n, oldNo: delta == null ? null : n - delta, text: newLines[n - 1] ?? '' });
  const dTop = topOld == null ? null : topNew - topOld;
  const dBot = botOld == null ? null : botNew - botOld;
  const before = Array.from({ length: extra.before }, (_, i) => ctx(topNew - extra.before + i, dTop));
  const after = Array.from({ length: extra.after }, (_, i) => ctx(botNew + 1 + i, dBot));
  return { ...h, lines: [...before, ...h.lines, ...after] };
}

const DECL = /^\s*(pub(\([^)]*\))?\s+)?((async|unsafe|const|export|default|static|extern\s+"\w+")\s+)*(fn|impl|struct|enum|trait|mod|def|function|class|func|interface)\b/;
const FUNCTION_KINDS = [6, 9, 12];

function indentOf(s: string) {
  return s.length - s.trimStart().length;
}

/** Heuristic for files without a language server: nearest declaration above, its end below. */
function guessRange(lines: string[], start: number, end: number, path: string): [number, number] | null {
  const python = path.endsWith('.py');
  const isDecl = path.endsWith('.cs') ? isCSharpDecl : (l: string) => DECL.test(l);
  let from = 0;
  for (let i = start; i >= Math.max(1, start - 500); i--) {
    if (isDecl(lines[i - 1] ?? '')) {
      from = i;
      break;
    }
  }
  if (!from) return null;
  const ind = indentOf(lines[from - 1]);
  for (let i = Math.max(end, from) + 1; i <= Math.min(lines.length, end + 800); i++) {
    const l = lines[i - 1];
    if (python ? l.trim() && indentOf(l) <= ind : indentOf(l) === ind && l.trimStart().startsWith('}')) return [from, python ? i - 1 : i];
  }
  return null;
}

/** Line range of the function (or other item) around new lines `start..end`. */
export async function enclosingRange(path: string, start: number, end: number, lines: string[]): Promise<[number, number] | null> {
  if (lspUsable(path)) {
    const syms = (await api.lspSymbols(path).catch(() => [])).filter((s) => s.start <= start && s.end >= end);
    const pick = (list: typeof syms) => list.sort((a, b) => a.end - a.start - (b.end - b.start))[0];
    const best = pick(syms.filter((s) => FUNCTION_KINDS.includes(s.kind))) ?? pick(syms);
    if (best) return [best.start, best.end];
  }
  return guessRange(lines, start, end, path);
}
