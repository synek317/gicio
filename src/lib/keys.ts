import type { Hunk, LineKey, Side } from './types';

/** String form of a changed line's identity, used as map/set key in the UI. */
export const keyOf = (path: string, side: Side, line: number) => `${side}:${line}:${path}`;

export function parseKey(k: string): LineKey {
  const i = k.indexOf(':');
  const j = k.indexOf(':', i + 1);
  return { side: k.slice(0, i) as Side, line: Number(k.slice(i + 1, j)), path: k.slice(j + 1) };
}

export function changedKeys(h: Hunk): string[] {
  const out: string[] = [];
  for (const l of h.lines) {
    if (l.kind === 'added' && l.newNo != null) out.push(keyOf(h.path, 'new', l.newNo));
    else if (l.kind === 'removed' && l.oldNo != null) out.push(keyOf(h.path, 'old', l.oldNo));
  }
  return out;
}
