import { api } from './api';
import { tokenize, type Tok } from './highlight';
import type { FileDiff, FileTexts } from './types';

interface Entry { sig: string; texts: FileTexts | null; old: Tok[][] | null; new: Tok[][] | null }

/** Full old/new file texts and their syntax tokens, reloaded whenever the file's hunks change. */
class TextCache {
  entries = $state.raw<Record<string, Entry>>({});

  private set(path: string, e: Entry) {
    this.entries = { ...this.entries, [path]: e };
  }

  ensure(file: FileDiff) {
    const sig = file.hunks.map((h) => h.id).join(',');
    if (this.entries[file.path]?.sig === sig || file.binary) return;
    this.set(file.path, { sig, texts: null, old: null, new: null });
    api
      .fileTexts(file.path)
      .then(async (texts) => {
        if (this.entries[file.path]?.sig !== sig) return;
        this.set(file.path, { sig, texts, old: null, new: null });
        const [o, n] = await Promise.all([tokenize(texts.old, file.oldPath ?? file.path), tokenize(texts.new, file.path)]);
        if (this.entries[file.path]?.sig === sig) this.set(file.path, { sig, texts, old: o, new: n });
      })
      .catch(() => {});
  }

  tokens(path: string, side: 'old' | 'new', line: number): Tok[] | null {
    const e = this.entries[path];
    return (side === 'old' ? e?.old : e?.new)?.[line - 1] ?? null;
  }
}

export const texts = new TextCache();
