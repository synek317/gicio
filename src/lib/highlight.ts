import { createHighlighter, type BundledLanguage, type Highlighter } from 'shiki';

export interface Tok { text: string; style: string; col: number }

const LANGS: Record<string, string> = {
  rs: 'rust', toml: 'toml', md: 'markdown', json: 'json', yml: 'yaml', yaml: 'yaml', ts: 'typescript', js: 'javascript',
  py: 'python', sh: 'shellscript', html: 'html', css: 'css', svelte: 'svelte', go: 'go', c: 'c', h: 'c', cpp: 'cpp',
};
const LIMIT = 1_000_000;

let highlighter: Promise<Highlighter> | null = null;

export function langFor(path: string): string | null {
  return LANGS[path.split('.').pop()?.toLowerCase() ?? ''] ?? null;
}

function plain(code: string): Tok[][] {
  return code.split('\n').map((text) => [{ text, style: '', col: 0 }]);
}

/** Tokens per line (index = line number - 1); whole-file tokenization keeps multi-line constructs right. */
export async function tokenize(code: string, path: string): Promise<Tok[][]> {
  const lang = langFor(path);
  if (!lang || code.length > LIMIT) return plain(code);
  highlighter ??= createHighlighter({ themes: ['github-light', 'github-dark'], langs: [...new Set(Object.values(LANGS))] });
  const h = await highlighter;
  const res = h.codeToTokens(code, { lang: lang as BundledLanguage, themes: { light: 'github-light', dark: 'github-dark' } });
  return res.tokens.map((line) => {
    let col = 0;
    return line.map((t) => {
      const style = t.htmlStyle ? Object.entries(t.htmlStyle).map(([k, v]) => `${k}:${v}`).join(';') : t.color ? `color:${t.color}` : '';
      const tok = { text: t.content, style, col };
      col += t.content.length;
      return tok;
    });
  });
}
