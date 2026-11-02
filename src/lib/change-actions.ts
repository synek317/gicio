import { run } from './actions';
import { api } from './api';
import { confirm } from './confirm';
import { app } from './state.svelte';

/** Keeps an AI change: the lines it wrote count as reviewed. */
export async function acceptChange(id: string) {
  if (await run(api.acceptChange(id))) app.toast('Kept. The lines the AI wrote count as reviewed.');
}

/** Undoes an AI change after asking: its files go back to how they were before. */
export async function rejectChange(id: string) {
  const c = app.snap?.changes.find((c) => c.id === id);
  if (!c || !(await confirm(`Undo the AI's changes in ${c.files.join(', ')}? The files go back to how they were before it.`, 'Undo AI changes'))) return;
  try {
    const [snap, skipped] = await api.rejectChange(id);
    app.applySnapshot(snap);
    app.toast(skipped.length ? `Undone, except ${skipped.join(', ')}: edited again since, so left as it is.` : 'Undone. The files are back to how they were before.', skipped.length ? 'error' : 'info');
  } catch (e) {
    app.toast(String(e), 'error');
  }
}
