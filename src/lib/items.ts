/** Code items (functions, structs, impls, classes, methods, inline mods, use groups, macros…) found
 *  in a file's text, so whole items can be accepted at once. Small brace-matching scanners: good
 *  enough for Rust, C# and other C-like languages, indentation-based for Python. */
import { csharp } from './items-csharp';
import { python } from './items-python';
import { rustLike } from './items-rust-like';
import type { CodeItem } from './item-scan';

export type { CodeItem } from './item-scan';

function scannerFor(path: string): (lines: string[]) => CodeItem[] {
  if (path.endsWith('.py')) return python;
  if (path.endsWith('.cs') || path.endsWith('.csx')) return csharp;
  return rustLike;
}

const cache = new Map<string, CodeItem[]>();

export function findItems(path: string, text: string): CodeItem[] {
  const key = `${path}\u0000${text.length}\u0000${text.slice(0, 200)}\u0000${text.slice(-200)}`;
  let items = cache.get(key);
  if (!items) {
    items = scannerFor(path)(text.split('\n'));
    if (cache.size > 100) cache.clear();
    cache.set(key, items);
  }
  return items;
}

export function itemLabel(it: CodeItem): string {
  const name = it.name.length > 28 ? `${it.name.slice(0, 27)}…` : it.name;
  return it.kind === 'use' ? `use ${name}` : `${it.kind} ${name}`.trim();
}
