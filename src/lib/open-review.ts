import { go, plan } from './actions';
import { api } from './api';
import { settings } from './app-settings.svelte';
import { toBackend, type BackendForm } from './backend-form';
import { baseOf, nothingToReview, stageOf, type StartOptions } from './start-options';
import { app } from './state.svelte';

/** Opens (or resumes) the review of `repo` with the given choices. Without `intent` (what the change
 *  should do) a resumed review keeps the one it has and a new one uses the commit messages. With
 *  `fresh` the saved review is moved aside instead of being resumed. */
export async function openReview(repo: string, o: StartOptions, backend: BackendForm, intent = '', fresh = false) {
  const snap = await api.openSession(repo, baseOf(o), toBackend(backend), stageOf(o), intent.trim() || null, fresh);
  if (snap.notice) app.toast(snap.notice, 'info', 15000);
  // From the start screen an empty review would be a dead end without any explanation.
  if (!app.snap && !snap.files.length) throw new Error(nothingToReview(o));
  if (!app.snap) app.layout = settings.value.prefs.layout;
  app.applySnapshot(snap);
  app.history = [];
  app.future = [];
  // Back where the reviewer left off; a note says how the change moved since then.
  const last = snap.chunks.find((c) => c.id === snap.lastChunk) ?? snap.chunks[0];
  if (last) go({ kind: 'chunk', id: last.id }, false);
  if (o.order === 'ai' && !snap.plannedByAi && snap.files.length) plan(false, o.steps);
}
