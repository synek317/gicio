import { changedKeys } from './keys';
import type { Hunk, MarkStatus } from './types';

export interface Progress { total: number; accepted: number; deferred: number; pending: number }

export const emptyProgress = (): Progress => ({ total: 0, accepted: 0, deferred: 0, pending: 0 });

export function hunkProgress(h: Hunk, marks: Map<string, MarkStatus>, into = emptyProgress()): Progress {
  for (const k of changedKeys(h)) {
    into.total++;
    const s = marks.get(k);
    if (s === 'accepted') into.accepted++;
    else if (s === 'deferred') into.deferred++;
    else into.pending++;
  }
  return into;
}

export function hunksProgress(hunks: Hunk[], marks: Map<string, MarkStatus>): Progress {
  const p = emptyProgress();
  for (const h of hunks) hunkProgress(h, marks, p);
  return p;
}

export type ProgressState = 'todo' | 'started' | 'deferred' | 'done';

export function progressState(p: Progress): ProgressState {
  if (p.pending === 0) return p.deferred > 0 ? 'deferred' : 'done';
  return p.accepted + p.deferred > 0 ? 'started' : 'todo';
}
