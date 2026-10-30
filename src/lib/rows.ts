import { keyOf } from './keys';
import type { Hunk, LineKind, Side } from './types';

export interface Cell { side: Side; no: number; text: string; kind: LineKind; key: string | null }
export interface UnifiedRow { cell: Cell; oldNo: number | null; newNo: number | null }
export interface SplitRow { left: Cell | null; right: Cell | null }

function cellsOf(h: Hunk) {
  return h.lines.map((l) => {
    const side: Side = l.kind === 'removed' ? 'old' : 'new';
    const no = (side === 'old' ? l.oldNo : l.newNo) ?? 0;
    const key = l.kind === 'context' ? null : keyOf(h.path, side, no);
    return { line: l, cell: { side, no, text: l.text, kind: l.kind, key } as Cell };
  });
}

export function unifiedRows(h: Hunk): UnifiedRow[] {
  return cellsOf(h).map(({ line, cell }) => ({ cell, oldNo: line.oldNo, newNo: line.newNo }));
}

/** Pairs runs of removed/added lines side by side, like kdiff3. */
export function splitRows(h: Hunk): SplitRow[] {
  const rows: SplitRow[] = [];
  let removed: Cell[] = [];
  let added: Cell[] = [];
  const flush = () => {
    for (let i = 0; i < Math.max(removed.length, added.length); i++) rows.push({ left: removed[i] ?? null, right: added[i] ?? null });
    removed = [];
    added = [];
  };
  for (const { line, cell } of cellsOf(h)) {
    if (line.kind === 'removed') {
      if (added.length) flush();
      removed.push(cell);
    } else if (line.kind === 'added') {
      added.push(cell);
    } else {
      flush();
      rows.push({ left: { ...cell, side: 'old', no: line.oldNo ?? 0 }, right: cell });
    }
  }
  flush();
  return rows;
}
