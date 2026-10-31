/** A file mention in free text: `src/a.rs`, `src/a.rs:42`, `src/a.rs:42-50`, `a.rs#L42-L50`.
 *  Only says what the text looks like; whether the file exists is decided elsewhere. */
export class PathRef {
  private constructor(
    readonly path: string,
    readonly line: number | null,
    readonly endLine: number | null,
  ) {}

  /** The whole string must be one mention (e.g. an inline code span). */
  static parse(raw: string): PathRef | null {
    const m = new RegExp(`^${PATTERN}$`).exec(raw.trim());
    return m ? PathRef.fromMatch(m) : null;
  }

  /** From a match of `PATTERN`; null when the captured path is not a usable file path. */
  static fromMatch(m: RegExpExecArray): PathRef | null {
    const path = m[1].replace(/^\.\//, '');
    if (!/^[\w./-]+$/.test(path) || path.endsWith('/')) return null;
    const line = num(m[2] ?? m[4]);
    const end = num(m[3] ?? m[5]);
    return new PathRef(path, line, line != null && end != null && end >= line ? end : null);
  }

  get isBareName(): boolean {
    return !this.path.includes('/');
  }

  get fileName(): string {
    return this.path.slice(this.path.lastIndexOf('/') + 1);
  }
}

export interface PathMatch {
  index: number;
  text: string;
  ref: PathRef;
}

/** Every mention in a piece of plain text, in order. */
export function findPathRefs(text: string): PathMatch[] {
  const out: PathMatch[] = [];
  for (const m of text.matchAll(new RegExp(`(?<![\\w./-])${PATTERN}(?![\\w/])`, 'g'))) {
    const ref = PathRef.fromMatch(m);
    if (ref) out.push({ index: m.index, text: m[0], ref });
  }
  return out;
}

const num = (s: string | undefined) => (s == null ? null : Number(s));

/** Optional `./` or `/`, directories, then a name with an extension starting with a letter
 *  (so version numbers like `1.5` don't match) or a dotfile, then an optional line or range. */
const PATTERN = String.raw`((?:\.?/)?(?:[\w.-]+/)*(?:[\w-][\w.-]*\.[A-Za-z]\w{0,9}|\.[A-Za-z][\w.-]*))(?::(\d+)(?:-(\d+))?(?::\d+)?|#L(\d+)(?:-L?(\d+))?)?`;
