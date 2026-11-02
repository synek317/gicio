import { rust } from '@codemirror/lang-rust';
import type { Extension } from '@codemirror/state';
import { csharp } from '@replit/codemirror-lang-csharp';

/** Syntax highlighting for the built-in editors, by file extension. */
export function languageFor(path: string): Extension[] {
  if (path.endsWith('.rs')) return [rust()];
  if (path.endsWith('.cs') || path.endsWith('.csx')) return [csharp()];
  return [];
}
