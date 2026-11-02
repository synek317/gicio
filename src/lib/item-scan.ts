/** Shared pieces of the code item scanners: a small brace matcher over comment- and string-free text. */
export interface CodeItem {
  kind: string;
  name: string;
  /** 1-based, inclusive line range (attributes and doc comments above are included). */
  start: number;
  end: number;
}

/** Strips strings, chars and line comments so brackets inside them don't count. */
function code(line: string): string {
  return line.replace(/\/\/.*$/, '').replace(/"(\\.|[^"\\])*"/g, '""').replace(/'(\\.|[^'\\])'/g, "''");
}

/** Line where the bracket group opened at/after `from` closes; `;` before any `{` ends a short item,
 *  unless it sits inside `(…)` or `[…]` (as in `-> [u8; 4]`). */
export function blockEnd(lines: string[], from: number, open = '{', close = '}', stopAtSemicolon = true): number {
  let depth = 0;
  let nested = 0;
  let opened = false;
  for (let i = from; i < lines.length && i < from + 5000; i++) {
    for (const ch of code(lines[i])) {
      if (ch === open) {
        depth++;
        opened = true;
      } else if (ch === close) {
        depth--;
        if (opened && depth === 0) return i;
      } else if (ch === '(' || ch === '[') nested++;
      else if (ch === ')' || ch === ']') nested--;
      else if (ch === ';' && !opened && depth === 0 && nested === 0 && stopAtSemicolon) return i;
    }
  }
  return from;
}

/** Attributes and doc comments (lines matching `preamble`) directly above an item belong to it. */
export function withPreamble(lines: string[], start: number, preamble: RegExp): number {
  let s = start;
  while (s > 0 && preamble.test(lines[s - 1])) s--;
  return s;
}

/** Consecutive import statements (`use`, `using`) form one group, named by their count. */
export function importGroups(lines: string[], statement: RegExp, kind: string, between: RegExp): CodeItem[] {
  const items: CodeItem[] = [];
  for (let i = 0; i < lines.length; i++) {
    if (!statement.test(lines[i])) continue;
    const start = i;
    let count = 0;
    while (i < lines.length && statement.test(lines[i])) {
      i = blockEnd(lines, i) + 1;
      count++;
      while (i < lines.length && between.test(lines[i])) i++;
    }
    items.push({ kind, name: `×${count}`, start: start + 1, end: i });
    i--;
  }
  return items;
}
