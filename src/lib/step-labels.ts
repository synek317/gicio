import type { Chunk } from './types';

export type StepEntry = { kind: 'step'; chunk: Chunk; label: string } | { kind: 'group'; title: string; label: string; steps: { chunk: Chunk; label: string }[] };

/** Steps in order; consecutive steps of one feature nest under it, numbered 2.1, 2.2… (a feature of
 *  one step stays flat). */
export function stepEntries(chunks: Chunk[]): StepEntry[] {
  const runs: { group: string | null; chunks: Chunk[] }[] = [];
  for (const c of chunks) {
    const last = runs.at(-1);
    if (last && c.group && last.group === c.group) last.chunks.push(c);
    else runs.push({ group: c.group ?? null, chunks: [c] });
  }
  return runs.map((r, i): StepEntry => {
    const label = String(i + 1);
    if (!r.group || r.chunks.length === 1) return { kind: 'step', chunk: r.chunks[0], label };
    return { kind: 'group', title: r.group, label, steps: r.chunks.map((chunk, j) => ({ chunk, label: `${label}.${j + 1}` })) };
  });
}

/** The number a step has in the step list. */
export function stepLabel(chunks: Chunk[], id: string): string | null {
  for (const e of stepEntries(chunks)) {
    if (e.kind === 'step' && e.chunk.id === id) return e.label;
    if (e.kind === 'group') for (const s of e.steps) if (s.chunk.id === id) return s.label;
  }
  return null;
}
