/** Code items (functions, structs, impls, inline mods, use groups, macros…) found in a file's
 *  text, so whole items can be accepted at once. A small brace-matching scanner: good enough for
 *  Rust and other C-like languages, indentation-based for Python. */
export interface CodeItem {
  kind: string;
  name: string;
  /** 1-based, inclusive line range (attributes and doc comments above are included). */
  start: number;
  end: number;
}

const RUST_DECL = /^\s*(pub(\([^)]*\))?\s+)?((async|unsafe|const|default|extern(\s+"[^"]*")?)\s+)*(fn|struct|enum|union|trait|impl|mod|type|const|static|macro_rules!)\s*([A-Za-z_]\w*)?/;
const C_DECL = /^\s*(export\s+)?(default\s+)?(async\s+)?(function|class|interface|func|type)\s+([A-Za-z_$][\w$]*)/;
const USE = /^\s*(pub(\([^)]*\))?\s+)?use\s/;
const MACRO_CALL = /^\s*([a-z_][\w:]*)!\s*[{([]/;
const PY_DECL = /^(\s*)(async\s+)?(def|class)\s+([A-Za-z_]\w*)/;

/** Strips strings, chars and line comments so brackets inside them don't count. */
function code(line: string): string {
  return line.replace(/\/\/.*$/, '').replace(/"(\\.|[^"\\])*"/g, '""').replace(/'(\\.|[^'\\])'/g, "''");
}

/** Line where the bracket group opened at/after `from` closes; `;` before any `{` ends a short item,
 *  unless it sits inside `(…)` or `[…]` (as in `-> [u8; 4]`). */
function blockEnd(lines: string[], from: number, open = '{', close = '}', stopAtSemicolon = true): number {
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

/** Attributes and doc comments directly above an item belong to it. */
function withPreamble(lines: string[], start: number): number {
  let s = start;
  while (s > 0 && /^\s*(#\[|#!\[|\/\/\/|\/\/!|@)/.test(lines[s - 1])) s--;
  return s;
}

function rustLike(lines: string[]): CodeItem[] {
  const items: CodeItem[] = [];
  for (let i = 0; i < lines.length; i++) {
    const l = lines[i];
    const rd = RUST_DECL.exec(l);
    const cd = rd ? null : C_DECL.exec(l);
    if (rd || cd) {
      const kind = rd ? rd[6].replace('!', '') : cd![4];
      const name = kind === 'impl' ? l.trim().replace(/\s*\{.*$/, '').replace(/^(pub\s+)?(unsafe\s+)?impl(<[^>]*>)?\s*/, '') : ((rd ? rd[7] : cd![5]) ?? '');
      items.push({ kind, name, start: withPreamble(lines, i) + 1, end: blockEnd(lines, i) + 1 });
      continue;
    }
    const m = MACRO_CALL.exec(l);
    if (m && m[1] !== 'macro_rules') {
      const open = l.slice(l.indexOf('!') + 1).trim()[0];
      const close = { '{': '}', '(': ')', '[': ']' }[open] ?? '}';
      const end = blockEnd(lines, i, open, close, false);
      // Item-level macros (`thread_local! {…}`, multi-line calls), not every `println!` in a body.
      if (end - i >= 2 || l.length === l.trimStart().length) items.push({ kind: 'macro', name: `${m[1]}!`, start: withPreamble(lines, i) + 1, end: end + 1 });
    }
  }
  // Consecutive `use` statements form one group.
  for (let i = 0; i < lines.length; i++) {
    if (!USE.test(lines[i])) continue;
    const start = i;
    let count = 0;
    while (i < lines.length && USE.test(lines[i])) {
      i = blockEnd(lines, i) + 1;
      count++;
      while (i < lines.length && /^\s*(#\[|\/\/)/.test(lines[i])) i++;
    }
    items.push({ kind: 'use', name: `×${count}`, start: start + 1, end: i });
    i--;
  }
  return items;
}

function python(lines: string[]): CodeItem[] {
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
    items.push({ kind: d[3], name: d[4], start: withPreamble(lines, i) + 1, end: end + 1 });
  });
  return items;
}

const cache = new Map<string, CodeItem[]>();

export function findItems(path: string, text: string): CodeItem[] {
  const key = `${path}\u0000${text.length}\u0000${text.slice(0, 200)}\u0000${text.slice(-200)}`;
  let items = cache.get(key);
  if (!items) {
    const lines = text.split('\n');
    items = path.endsWith('.py') ? python(lines) : rustLike(lines);
    if (cache.size > 100) cache.clear();
    cache.set(key, items);
  }
  return items;
}

export function itemLabel(it: CodeItem): string {
  const name = it.name.length > 28 ? `${it.name.slice(0, 27)}…` : it.name;
  return it.kind === 'use' ? `use ${name}` : `${it.kind} ${name}`.trim();
}
