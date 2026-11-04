import { DEFAULT_STEP_COUNT } from './step-count';
import type { Snapshot, StepCount } from './types';

/** Everything the start screen asks besides the repository and the AI. */
export interface StartOptions {
  scope: 'uncommitted' | 'since';
  base: string;
  stage: 'memory' | 'index';
  order: 'ai' | 'files';
  /** How the AI splits the change; only used with the `ai` order. */
  steps: StepCount;
}

export const defaultStartOptions = (): StartOptions => ({ scope: 'uncommitted', base: 'main', stage: 'memory', order: 'ai', steps: DEFAULT_STEP_COUNT });

export function baseOf(o: StartOptions): string {
  return o.scope === 'uncommitted' ? 'HEAD' : o.base;
}

/** Staging only makes sense for uncommitted changes (the index sits on top of HEAD). */
export function stageOf(o: StartOptions): 'memory' | 'index' {
  return o.scope === 'uncommitted' ? o.stage : 'memory';
}

export function optionsOf(s: Snapshot): StartOptions {
  const uncommitted = s.baseRef === 'HEAD';
  return { scope: uncommitted ? 'uncommitted' : 'since', base: uncommitted ? 'main' : s.baseRef, stage: s.stageMode, order: s.plannedByAi ? 'ai' : 'files', steps: s.stepCount };
}
