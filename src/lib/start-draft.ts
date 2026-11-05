import { fromBackend, type BackendForm } from './backend-form';
import { optionsOf, type StartOptions } from './start-options';
import type { Snapshot } from './types';

/** What the start screen is filled in with when coming back from a review that was started over. */
export interface StartDraft {
  repo: string;
  options: StartOptions;
  form: BackendForm;
  /** The reviewer's own description; one taken from the commit messages comes back by itself. */
  intent: string;
}

let draft: StartDraft | null = null;

/** A review without an AI plan may just have lost it (planning failed or is still running), which
 *  says nothing about the order wanted now: then `defaultOrder` decides. */
export function draftOf(s: Snapshot, defaultOrder: StartOptions['order']): StartDraft {
  const options = { ...optionsOf(s), order: s.plannedByAi || s.planning ? ('ai' as const) : defaultOrder };
  return { repo: s.repo, options, form: fromBackend(s.backend), intent: s.intent?.source === 'reviewer' ? s.intent.text : '' };
}

export function stashStartDraft(d: StartDraft) {
  draft = d;
}

/** Hands the draft out once: other ways to the start screen must not see it. */
export function takeStartDraft(): StartDraft | null {
  const d = draft;
  draft = null;
  return d;
}
