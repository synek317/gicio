import { settings } from './app-settings.svelte';
import { changedKeys } from './keys';
import { app } from './state.svelte';

/** Lines with nothing to review: empty, or only brackets and punctuation (`}`, `{`, `});`, `],`). */
export function isTrivialLine(text: string): boolean {
  return /^[\s{}()[\];,.]*$/.test(text);
}

/**
 * After accepting, files where only blank lines are still pending: accept them right away or ask,
 * depending on Settings → Preferences.
 */
export function blankLeftovers(paths: Iterable<string>): { path: string; keys: string[] } | null {
  for (const path of paths) {
    const file = app.files.get(path);
    if (!file) continue;
    const blank: string[] = [];
    let acceptedText = 0;
    let open = false;
    for (const h of file.hunks) {
      const keys = changedKeys(h);
      const lines = h.lines.filter((l) => l.kind !== 'context');
      keys.forEach((k, i) => {
        const isBlank = isTrivialLine(lines[i].text);
        const status = app.marks.get(k);
        if (status === 'accepted') {
          if (!isBlank) acceptedText++;
        } else if (isBlank && status === undefined) blank.push(k);
        else if (!isBlank || status === 'deferred') open = true;
      });
    }
    if (!open && blank.length && acceptedText) return { path, keys: blank };
  }
  return null;
}

export function afterAccept(paths: Set<string>, accept: (keys: string[]) => void) {
  const pref = settings.value.prefs.blankLines;
  if (pref === 'never' || app.blankPrompt) return;
  const hit = blankLeftovers(paths);
  if (!hit) return;
  if (pref === 'accept') accept(hit.keys);
  else app.blankPrompt = hit;
}
