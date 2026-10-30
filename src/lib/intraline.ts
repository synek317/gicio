/** Character ranges `[from, to)` that differ between two similar lines (word-level diff). */
export interface IntraLine { old: [number, number][]; new: [number, number][] }

const TOKEN = /\w+|\s+|[^\w\s]/g;
const MAX_TOKENS = 300;

function tokens(s: string): { t: string; at: number }[] {
  return [...s.matchAll(TOKEN)].map((m) => ({ t: m[0], at: m.index ?? 0 }));
}

function ranges(toks: { t: string; at: number }[], keep: boolean[]): [number, number][] {
  const out: [number, number][] = [];
  toks.forEach((tok, i) => {
    if (keep[i]) return;
    const last = out[out.length - 1];
    if (last && last[1] === tok.at) last[1] = tok.at + tok.t.length;
    else out.push([tok.at, tok.at + tok.t.length]);
  });
  return out;
}

/** Null when the lines are too different for word highlights to help. */
export function intraLine(a: string, b: string): IntraLine | null {
  const ta = tokens(a);
  const tb = tokens(b);
  if (!ta.length || !tb.length || ta.length > MAX_TOKENS || tb.length > MAX_TOKENS) return null;
  // LCS over tokens.
  const dp = Array.from({ length: ta.length + 1 }, () => new Uint16Array(tb.length + 1));
  for (let i = ta.length - 1; i >= 0; i--)
    for (let j = tb.length - 1; j >= 0; j--) dp[i][j] = ta[i].t === tb[j].t ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1]);
  const ka = new Array(ta.length).fill(false);
  const kb = new Array(tb.length).fill(false);
  for (let i = 0, j = 0; i < ta.length && j < tb.length; ) {
    if (ta[i].t === tb[j].t) {
      ka[i++] = true;
      kb[j++] = true;
    } else if (dp[i + 1][j] >= dp[i][j + 1]) i++;
    else j++;
  }
  const same = ta.filter((_, i) => ka[i] && ta[i].t.trim()).length;
  const words = Math.max(ta.filter((t) => t.t.trim()).length, tb.filter((t) => t.t.trim()).length);
  if (!words || same / words < 0.35) return null;
  return { old: ranges(ta, ka), new: ranges(tb, kb) };
}
