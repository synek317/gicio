import { withPreamble, type CodeItem } from './item-scan';

const PY_DECL = /^(\s*)(async\s+)?(def|class)\s+([A-Za-z_]\w*)/;
const PREAMBLE = /^\s*@/;

/** Functions and classes, by indentation. */
export function python(lines: string[]): CodeItem[] {
  const items: CodeItem[] = [];
  lines.forEach((l, i) => {
    const d = PY_DECL.exec(l);
    if (!d) return;
    const indent = d[1].length;
    let end = i;
    for (let j = i + 1; j < lines.length; j++) {
      if (!lines[j].trim()) continue;
      if (lines[j].length - lines[j].trimStart().length <= indent) break;
      end = j;
    }
    items.push({ kind: d[3], name: d[4], start: withPreamble(lines, i, PREAMBLE) + 1, end: end + 1 });
  });
  return items;
}
