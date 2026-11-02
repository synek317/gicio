import { intraLine } from './intraline';
import { keyOf } from './keys';
import type { DiffLine, Hunk, LineKind, Side } from './types';

/** `block`: keys of the run of changed lines this line belongs to, accepted together from the gutter. */
export interface Cell { side: Side; no: number; text: string; kind: LineKind; key: string | null; hl?: [number, number][]; block?: string[] }
export interface UnifiedRow { cell: Cell; oldNo: number | null; newNo: number | null }
export interface SplitRow { left: Cell | null; right: Cell | null }

function cellsOf(h: Hunk) {
  let block: string[] = [];
  return h.lines.map((l, i) => {
    const side: Side = l.kind === 'removed' ? 'old' : 'new';
    const no = (side === 'old' ? l.oldNo : l.newNo) ?? 0;
    const key = l.kind === 'context' ? null : keyOf(h.path, side, no);
    if (key && (i === 0 || h.lines[i - 1].kind === 'context')) block = [];
    if (key) block.push(key);
    return { line: l, cell: { side, no, text: l.text, kind: l.kind, key, block: key ? block : undefined } as Cell };
  });
}

/** Pairs each run of removed lines with the following run of added lines, i-th with i-th. */
function pairs(lines: DiffLine[]): [number, number][] {
  const out: [number, number][] = [];
  for (let i = 0; i < lines.length; ) {
    if (lines[i].kind !== 'removed') {
      i++;
      continue;
    }
    const rs = i;
    while (i < lines.length && lines[i].kind === 'removed') i++;
    const as = i;
    while (i < lines.length && lines[i].kind === 'added') i++;
    for (let k = 0; k < Math.min(as - rs, i - as); k++) out.push([rs + k, as + k]);
  }
  return out;
}

function withHighlights(h: Hunk) {
  const cells = cellsOf(h);
  for (const [r, a] of pairs(h.lines)) {
    const d = intraLine(h.lines[r].text, h.lines[a].text);
    if (d) {
      cells[r].cell.hl = d.old;
      cells[a].cell.hl = d.new;
    }
  }
  return cells;
}

export function unifiedRows(h: Hunk): UnifiedRow[] {
  return withHighlights(h).map(({ line, cell }) => ({ cell, oldNo: line.oldNo, newNo: line.newNo }));
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
  for (const { line, cell } of withHighlights(h)) {
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
