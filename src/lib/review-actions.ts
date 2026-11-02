import { api, inTauri } from './api';
import { run } from './actions';
import { app } from './state.svelte';
import type { ReviewScope, Tool } from './types';

export async function reviewWithAi(scope: ReviewScope, focus: string | null = null) {
  try {
    await api.aiReview(scope, focus);
    app.rightTab = 'findings';
    app.toast('AI review started. Findings will appear in the Findings tab and next to the code.');
  } catch (e) {
    app.toast(String(e), 'error');
  }
}

export async function runTool(tool: Tool) {
  try {
    const msg = await api.runTool(tool);
    app.toast(msg);
    if (tool === 'clippy' || tool === 'dotnetBuild') app.rightTab = 'findings';
  } catch (e) {
    app.toast(String(e), 'error');
  }
}

/** Copies the review as markdown and, in the app, offers to save it as a file. */
export async function exportReview() {
  try {
    const md = await api.exportReview();
    await navigator.clipboard.writeText(md).catch(() => {});
    if (inTauri) {
      const { save } = await import('@tauri-apps/plugin-dialog');
      const path = await save({ defaultPath: 'review.md', filters: [{ name: 'Markdown', extensions: ['md'] }] });
      if (path) {
        await api.writeTextFile(path, md);
        return app.toast(`Saved to ${path} (also copied to the clipboard)`);
      }
    }
    app.toast('Review copied to the clipboard as markdown.');
  } catch (e) {
    app.toast(String(e), 'error');
  }
}

export async function resolveLater() {
  if (await run(api.resolveLater())) app.toast('The AI is working through everything you left for later.');
}
