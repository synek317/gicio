import { editFile, revealLine, run } from './actions';
import { api } from './api';
import { showPanel } from './panels.svelte';
import { app } from './state.svelte';
import type { CodeRef, Requirement } from './types';

/** Partly done or missing, and nobody has decided or done anything about it yet. */
export const needsWork = (r: Requirement) => r.status === 'open' && (r.verdict === 'missing' || r.verdict === 'partial');

/** What the AI pointed at: the diff when it is part of the change, else the file. */
export function showCode(ref: CodeRef) {
  if (app.files.has(ref.path)) revealLine(ref.path, 'new', ref.line);
  else editFile(ref.path, ref.line);
}

/** A new conversation about a requirement: on the code it points at, else about the whole change. */
export function discussRequirement(r: Requirement) {
  const at = r.evidence.find((e) => app.files.has(e.path));
  app.composer = {
    anchor: at ? { path: at.path, side: 'new', start: at.line, end: at.line } : null,
    chunk: at && app.view?.kind === 'chunk' ? app.view.id : null,
  };
  app.composerDraft = `${r.text}${r.note ? `\n\n${r.note}` : ''}`;
  app.convScope = 'here';
  showPanel('right');
  app.rightTab = 'conversations';
}

export async function implementRequirements(rs: Requirement[]) {
  if (await run(api.implementRequirements(rs.map((r) => r.id)))) app.toast('The AI is working on it. Its changes will land as a step at the end.');
}
