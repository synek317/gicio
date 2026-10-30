import type { Hunk, HunkRef } from './types';

/** The part of a hunk a chunk refers to, as a hunk of its own (id gets a `#from` suffix). */
export function sliceHunk(h: Hunk, ref: HunkRef): Hunk {
  const p = ref.part;
  if (!p) return h;
  const lines = h.lines.slice(p.from, p.to);
  const olds = lines.map((l) => l.oldNo).filter((n): n is number => n != null);
  const news = lines.map((l) => l.newNo).filter((n): n is number => n != null);
  return {
    ...h,
    id: `${h.id}#${p.from}`,
    baseId: h.id,
    lines,
    oldStart: olds[0] ?? h.oldStart,
    oldLen: olds.length,
    newStart: news[0] ?? h.newStart,
    newLen: news.length,
  };
}
