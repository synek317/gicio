import type { Progress, ProgressState } from './progress';

export const progressIcons: Record<ProgressState, string> = { todo: '○', started: '◐', deferred: '⏸', done: '✓' };

const labels: Record<ProgressState, string> = {
  todo: 'Not started',
  started: 'In progress',
  deferred: 'Rest put off for later',
  done: 'Done',
};

/** Tooltip for a progress icon: what the state means and how many lines are in each bucket. */
export function progressTooltip(st: ProgressState, p: Progress): string {
  const parts = [`${p.accepted} accepted`];
  if (p.deferred) parts.push(`${p.deferred} for later`);
  parts.push(`${p.pending} pending`);
  return `${labels[st]}: ${parts.join(', ')} of ${p.total} lines`;
}
