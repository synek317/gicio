/** How text is matched, shared by the find bar and the project search. */
export interface FindOptions {
  caseSensitive: boolean;
  wholeWord: boolean;
  regex: boolean;
}

export interface FindSpec extends FindOptions {
  query: string;
}

const WORD = /[\p{L}\p{N}_]/u;

/** The query as a global regular expression; throws a SyntaxError for an invalid regex. */
export function compileFind(spec: FindSpec): RegExp {
  const source = spec.regex ? spec.query : spec.query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  return new RegExp(source, spec.caseSensitive ? 'g' : 'gi');
}

/** `[from, to)` of the non-empty matches in one line. With whole words a match must not touch a
 *  letter, digit or `_` on either side (mirrors `gicio_core::search::Matcher`). */
export function matchRanges(text: string, re: RegExp, wholeWord: boolean): [number, number][] {
  const out: [number, number][] = [];
  re.lastIndex = 0;
  for (let m = re.exec(text); m; m = re.exec(text)) {
    const [from, to] = [m.index, m.index + m[0].length];
    if (from === to || (wholeWord && (WORD.test(text[from - 1] ?? '') || WORD.test(text[to] ?? '')))) {
      re.lastIndex = from + 1;
      continue;
    }
    out.push([from, to]);
  }
  return out;
}
