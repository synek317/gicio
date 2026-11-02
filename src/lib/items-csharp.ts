import { blockEnd, importGroups, withPreamble, type CodeItem } from './item-scan';

/** Modifiers that can open a member declaration (not `new` or `ref`: they also start statements). */
const MODIFIERS = '(?:public|private|protected|internal|static|abstract|sealed|partial|readonly|unsafe|file|virtual|override|async|extern|required|const|volatile)';
/** Attributes written on the declaration's own line: `[Fact] public void Works()`. */
const ATTRS = '(?:\\[[^\\]]*\\]\\s*)*';
/** A type as written before a member's name: `Task<List<int>>`, `int[]`, `(int, string)`, `string?`. */
const TYPE = '[\\w.<>\\[\\],?()\\s]+?';
/** Type parameters after a method's name: `<TKey, TValue>`. */
const TYPE_PARAMS = '(?:<[\\w\\s,.?<>\\[\\]]*>)?';

const TYPE_DECL = new RegExp(`^\\s*${ATTRS}(?:${MODIFIERS}\\s+|new\\s+|ref\\s+)*(class|struct|interface|enum|record(?:\\s+(?:class|struct))?|namespace|delegate\\s+${TYPE})\\s+([A-Za-z_][\\w.]*)`);
/** Methods and constructors: at least one modifier, an optional return type, the name, then `(` or `<`. */
const METHOD = new RegExp(`^\\s*${ATTRS}(?:${MODIFIERS}\\s+)+(${TYPE}\\s+)?([A-Za-z_]\\w*)\\s*${TYPE_PARAMS}\\s*\\(`);
const PROPERTY = new RegExp(`^\\s*${ATTRS}(?:${MODIFIERS}\\s+)+${TYPE}\\s+([A-Za-z_]\\w*)\\s*(\\{|=>)`);
const USING = /^\s*(global\s+)?using\s+(static\s+)?[\w.]+\s*(=\s*[^;]+)?;\s*(\/\/.*)?$/;
const PREAMBLE = /^\s*(\[|\/\/\/)/;

/** Whether a line starts a C# declaration (type, method, constructor or property). */
export function isCSharpDecl(line: string): boolean {
  return TYPE_DECL.test(line) || METHOD.test(line) || PROPERTY.test(line);
}

function declAt(line: string): { kind: string; name: string } | null {
  const t = TYPE_DECL.exec(line);
  if (t) {
    // A file-scoped `namespace X;` spans the whole file: not an item worth accepting at once.
    if (t[1] === 'namespace' && /;\s*$/.test(line)) return null;
    return { kind: t[1].startsWith('delegate') ? 'delegate' : t[1].replace(/\s+/g, ' '), name: t[2] };
  }
  const m = METHOD.exec(line);
  if (m) return { kind: m[1] ? 'method' : 'ctor', name: m[2] };
  const p = PROPERTY.exec(line);
  return p ? { kind: 'property', name: p[1] } : null;
}

/** Namespaces, types, methods, constructors, properties and `using` groups. */
export function csharp(lines: string[]): CodeItem[] {
  const items: CodeItem[] = [];
  lines.forEach((l, i) => {
    const d = declAt(l);
    if (d) items.push({ ...d, start: withPreamble(lines, i, PREAMBLE) + 1, end: blockEnd(lines, i) + 1 });
  });
  return [...items, ...importGroups(lines, USING, 'using', /^\s*\/\//)];
}
