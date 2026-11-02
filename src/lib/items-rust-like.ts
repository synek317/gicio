import { blockEnd, importGroups, withPreamble, type CodeItem } from './item-scan';

const RUST_DECL = /^\s*(pub(\([^)]*\))?\s+)?((async|unsafe|const|default|extern(\s+"[^"]*")?)\s+)*(fn|struct|enum|union|trait|impl|mod|type|const|static|macro_rules!)\s*([A-Za-z_]\w*)?/;
const C_DECL = /^\s*(export\s+)?(default\s+)?(async\s+)?(function|class|interface|func|type)\s+([A-Za-z_$][\w$]*)/;
const USE = /^\s*(pub(\([^)]*\))?\s+)?use\s/;
const MACRO_CALL = /^\s*([a-z_][\w:]*)!\s*[{([]/;
const PREAMBLE = /^\s*(#\[|#!\[|\/\/\/|\/\/!|@)/;

/** Rust, plus the declarations of other brace languages (TypeScript, Go…) that look alike. */
export function rustLike(lines: string[]): CodeItem[] {
  const items: CodeItem[] = [];
  for (let i = 0; i < lines.length; i++) {
    const l = lines[i];
    const rd = RUST_DECL.exec(l);
    const cd = rd ? null : C_DECL.exec(l);
    if (rd || cd) {
      const kind = rd ? rd[6].replace('!', '') : cd![4];
      const name = kind === 'impl' ? l.trim().replace(/\s*\{.*$/, '').replace(/^(pub\s+)?(unsafe\s+)?impl(<[^>]*>)?\s*/, '') : ((rd ? rd[7] : cd![5]) ?? '');
      items.push({ kind, name, start: withPreamble(lines, i, PREAMBLE) + 1, end: blockEnd(lines, i) + 1 });
      continue;
    }
    const m = MACRO_CALL.exec(l);
    if (m && m[1] !== 'macro_rules') {
      const open = l.slice(l.indexOf('!') + 1).trim()[0];
      const close = { '{': '}', '(': ')', '[': ']' }[open] ?? '}';
      const end = blockEnd(lines, i, open, close, false);
      // Item-level macros (`thread_local! {…}`, multi-line calls), not every `println!` in a body.
      if (end - i >= 2 || l.length === l.trimStart().length) items.push({ kind: 'macro', name: `${m[1]}!`, start: withPreamble(lines, i, PREAMBLE) + 1, end: end + 1 });
    }
  }
  return [...items, ...importGroups(lines, USE, 'use', /^\s*(#\[|\/\/)/)];
}
