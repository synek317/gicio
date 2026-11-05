import type { StepCount } from './types';

/** Mirrors `gicio_core::ai::StepWish::MAX_CHARS`. */
export const MAX_STEP_WISH = 200;

export const STEP_PRESETS = [
  { kind: 'few', label: 'Few large', hint: 'About 15-20 minutes of review per step' },
  { kind: 'balanced', label: 'Balanced', hint: 'About 5 minutes of review per step' },
  { kind: 'many', label: 'Many small', hint: 'About 1-2 minutes of review per step' },
] as const;

export const DEFAULT_STEP_COUNT: StepCount = { kind: 'balanced' };

/** False while the reviewer has picked Custom but not said what they want yet. */
export function stepCountReady(s: StepCount): boolean {
  return s.kind !== 'custom' || s.wish.trim() !== '';
}

export function sameStepCount(a: StepCount, b: StepCount): boolean {
  if (a.kind === 'custom' && b.kind === 'custom') return a.wish.trim() === b.wish.trim();
  return a.kind === b.kind;
}

/** Settings saved by older versions can hold an exact number of steps. */
export function upgradeStepCount(saved: StepCount | { kind: 'exact'; steps: number }): StepCount {
  if (saved.kind !== 'exact') return saved;
  return { kind: 'custom', wish: `${saved.steps} ${saved.steps === 1 ? 'step' : 'steps'}` };
}
