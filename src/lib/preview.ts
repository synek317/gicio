import { app } from './state.svelte';

/** Highlights the lines an accept button would accept, while the pointer is on it. */
export function previewOn(keys: string[]) {
  app.preview = new Set(keys.filter((k) => app.marks.get(k) !== 'accepted'));
}

export function previewOff() {
  if (app.preview.size) app.preview = new Set();
}
