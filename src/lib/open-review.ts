import { go, plan } from './actions';
import { api } from './api';
import { settings } from './app-settings.svelte';
import { toBackend, type BackendForm } from './backend-form';
import { baseOf, stageOf, type StartOptions } from './start-options';
import { app } from './state.svelte';

/** Opens (or resumes) the review of `repo` with the given choices. */
export async function openReview(repo: string, o: StartOptions, backend: BackendForm) {
  const snap = await api.openSession(repo, baseOf(o), toBackend(backend), stageOf(o));
  if (!app.snap) app.layout = settings.value.prefs.layout;
  app.applySnapshot(snap);
  app.history = [];
  app.future = [];
  // Back where the reviewer left off; a note says how the change moved since then.
  const last = snap.chunks.find((c) => c.id === snap.lastChunk) ?? snap.chunks[0];
  if (last) go({ kind: 'chunk', id: last.id }, false);
  if (snap.notice) app.toast(snap.notice, 'info', 15000);
  if (o.order === 'ai' && !snap.plannedByAi && snap.files.length) plan(false);
}
