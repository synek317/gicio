/** Languages the AI can write in, by English name (that is how prompts name them). */
const CODES = [
  'ar', 'bg', 'bn', 'ca', 'cs', 'da', 'de', 'el', 'es', 'et', 'fa', 'fi', 'fr', 'he', 'hi', 'hr', 'hu', 'id', 'it', 'ja', 'ko',
  'lt', 'lv', 'ms', 'nb', 'nl', 'pl', 'pt', 'ro', 'ru', 'sk', 'sl', 'sr', 'sv', 'th', 'tr', 'uk', 'vi', 'zh',
];

function englishName(code: string): string | null {
  try {
    return new Intl.DisplayNames(['en'], { type: 'language' }).of(code) ?? null;
  } catch {
    return null;
  }
}

/** The system's language, e.g. "Polish"; English when it cannot be told. */
export function systemLanguage(): string {
  const code = typeof navigator === 'undefined' ? 'en' : navigator.language.split('-')[0];
  return englishName(code) ?? 'English';
}

/** The system language first, then English, then the rest alphabetically. */
export function languageOptions(): string[] {
  const rest = CODES.map(englishName).filter((n): n is string => !!n).sort();
  return [...new Set([systemLanguage(), 'English', ...rest])];
}
