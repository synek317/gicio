import { findItems, type CodeItem } from './items';
import { keyOf } from './keys';
import type { FileDiff } from './types';

export interface ItemTarget { item: CodeItem; side: 'old' | 'new'; keys: string[] }

/** Changed lines of `file` inside a new-side item (removed lines count where they sit between its lines). */
function newSideKeys(file: FileDiff, it: CodeItem): string[] {
  const keys: string[] = [];
  for (const h of file.hunks) {
    let next = h.newStart; // the new line a removed line sits before
    for (const l of h.lines) {
      if (l.newNo != null) next = l.newNo;
      if (l.kind === 'added' && l.newNo != null && l.newNo >= it.start && l.newNo <= it.end) keys.push(keyOf(file.path, 'new', l.newNo));
      if (l.kind === 'removed' && l.oldNo != null && next >= it.start && next <= it.end) keys.push(keyOf(file.path, 'old', l.oldNo));
      if (l.newNo != null) next = l.newNo + 1;
    }
  }
  return keys;
}

function oldSideKeys(file: FileDiff, it: CodeItem): string[] {
  return file.hunks.flatMap((h) => h.lines).filter((l) => l.kind === 'removed' && l.oldNo != null && l.oldNo >= it.start && l.oldNo <= it.end).map((l) => keyOf(file.path, 'old', l.oldNo!));
}

/**
 * Items touched by the given new/old line numbers, innermost first, with the changed lines that
 * accepting each would cover. Items that only exist in the old text (deleted code) come last.
 */
export function itemsAround(file: FileDiff, texts: { old: string; new: string }, newLines: number[], oldLines: number[]): ItemTarget[] {
  const out: ItemTarget[] = [];
  const newItems = findItems(file.path, texts.new);
  const hits = (it: CodeItem, ns: number[]) => ns.some((n) => n >= it.start && n <= it.end);
  for (const it of newItems.filter((it) => hits(it, newLines)).sort((a, b) => a.end - a.start - (b.end - b.start))) {
    const keys = newSideKeys(file, it);
    if (keys.length) out.push({ item: it, side: 'new', keys });
  }
  const names = new Set(newItems.map((it) => `${it.kind} ${it.name}`));
  for (const it of findItems(file.path, texts.old).filter((it) => hits(it, oldLines) && !names.has(`${it.kind} ${it.name}`))) {
    const keys = oldSideKeys(file, it);
    if (keys.length) out.push({ item: it, side: 'old', keys });
  }
  return out;
}

const startsCache = new Map<string, Map<string, ItemTarget[]>>();

/** `${side}|${line}` of each item's first line -> the item, for inline accept buttons. */
export function itemStarts(file: FileDiff, texts: { old: string; new: string }): Map<string, ItemTarget[]> {
  const key = `${file.path}\u0000${file.hunks.map((h) => h.id).join(',')}\u0000${texts.new.length}\u0000${texts.old.length}`;
  const hit = startsCache.get(key);
  if (hit) return hit;
  const changedNew = file.hunks.flatMap((h) => h.lines).filter((l) => l.newNo != null).map((l) => l.newNo!);
  const changedOld = file.hunks.flatMap((h) => h.lines).filter((l) => l.kind === 'removed').map((l) => l.oldNo!);
  const map = new Map<string, ItemTarget[]>();
  for (const t of itemsAround(file, texts, changedNew, changedOld)) {
    const k = `${t.side}|${t.item.start}`;
    map.set(k, [...(map.get(k) ?? []), t]);
  }
  if (startsCache.size > 200) startsCache.clear();
  startsCache.set(key, map);
  return map;
}
