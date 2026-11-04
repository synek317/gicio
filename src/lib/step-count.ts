import type { StepCount } from './types';

/** Mirrors `gicio_core::ai::StepTotal::MAX`. */
export const MAX_STEP_TOTAL = 999;

export const STEP_PRESETS = [
  { kind: 'few', label: 'Few large', hint: 'About 15-20 minutes of review per step' },
  { kind: 'balanced', label: 'Balanced', hint: 'About 5 minutes of review per step' },
  { kind: 'many', label: 'Many small', hint: 'About 1-2 minutes of review per step' },
] as const;

export const DEFAULT_STEP_COUNT: StepCount = { kind: 'balanced' };
export const DEFAULT_STEP_TOTAL = 10;

/** The number typed by the reviewer, or null while it is not a valid number of steps. */
export function parseStepTotal(raw: string): number | null {
  if (!/^\d+$/.test(raw.trim())) return null;
  const n = Number(raw);
  return n >= 1 && n <= MAX_STEP_TOTAL ? n : null;
}

export function sameStepCount(a: StepCount, b: StepCount): boolean {
  if (a.kind === 'exact' && b.kind === 'exact') return a.steps === b.steps;
  return a.kind === b.kind;
}
