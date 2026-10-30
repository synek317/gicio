<script lang="ts">
  import type { Tok } from '../highlight';
  let { tokens, text, hl = undefined }: { tokens: Tok[] | null; text: string; hl?: [number, number][] } = $props();

  interface Piece { text: string; style: string; col: number; changed: boolean }

  /** Tokens cut at highlight boundaries, so changed words can be marked inside a line. */
  const pieces = $derived.by((): Piece[] => {
    const toks = tokens ?? [{ text, style: '', col: 0 }];
    if (!hl?.length) return toks.map((t) => ({ ...t, changed: false }));
    const out: Piece[] = [];
    for (const t of toks) {
      const cuts = new Set([t.col, t.col + t.text.length]);
      for (const [a, b] of hl) {
        if (a > t.col && a < t.col + t.text.length) cuts.add(a);
        if (b > t.col && b < t.col + t.text.length) cuts.add(b);
      }
      const points = [...cuts].sort((x, y) => x - y);
      for (let i = 0; i < points.length - 1; i++) {
        const [from, to] = [points[i], points[i + 1]];
        out.push({ text: t.text.slice(from - t.col, to - t.col), style: t.style, col: from, changed: hl.some(([a, b]) => from >= a && to <= b) });
      }
    }
    return out;
  });
</script>

{#each pieces as p}<span data-col={p.col} style={p.style} class:ichg={p.changed}>{p.text}</span>{/each}
