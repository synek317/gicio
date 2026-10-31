import { api } from './api';
import { acceptRestAndNext, back, editFocused, forward, go, markTarget, moveChunk, moveFocus, nextPendingChunk, openAiTask, openComposer, run } from './actions';
import { settings } from './app-settings.svelte';
import { jumpFile, toggleCurrentFile } from './files-nav';
import { comboOf, type ActionId } from './keymap';
import { app } from './state.svelte';

const handlers: Record<ActionId, () => void> = {
  nextHunk: () => moveFocus(1),
  prevHunk: () => moveFocus(-1),
  nextChunk: () => moveChunk(1),
  prevChunk: () => moveChunk(-1),
  nextPending: () => nextPendingChunk(),
  nextFile: () => jumpFile(1),
  prevFile: () => jumpFile(-1),
  collapseFile: () => toggleCurrentFile(),
  accept: () => markTarget('accepted'),
  defer: () => markTarget('deferred'),
  reset: () => markTarget(null),
  acceptRest: () => acceptRestAndNext(),
  comment: () => openComposer(),
  aiTask: () => openAiTask(),
  rename: () => {
    if (app.pointerCode) app.rename = { ...app.pointerCode };
    else app.toast('Point at a symbol (rust-analyzer running) and press F2, or right-click it.');
  },
  openFile: () => editFocused(),
  toggleLayout: () => (app.layout = app.layout === 'split' ? 'unified' : 'split'),
  laterView: () => go({ kind: 'later' }),
  undo: () => run(api.undo()),
  reload: () => run(api.refresh()),
  back: () => back(),
  forward: () => forward(),
  chat: () => (app.rightTab = 'chat'),
  settings: () => (settings.open = true),
  help: () => (app.showHelp = !app.showHelp),
};

/** Actions that also work while typing (in the editor or a text box). */
const WHILE_TYPING: ActionId[] = ['back', 'forward', 'settings'];

export function actionFor(combo: string): ActionId | undefined {
  const kb = settings.value.keybindings;
  return (Object.keys(kb) as ActionId[]).find((id) => kb[id]?.includes(combo));
}

export function handleKey(e: KeyboardEvent) {
  if (settings.capturing) return;
  const t = e.target as HTMLElement;
  const typing = t.closest('input, textarea, select, [contenteditable]');
  if (e.key === 'Escape') {
    if (typing) t.blur();
    if (app.menu) app.menu = null;
    else if (app.showHelp) app.showHelp = false;
    else if (app.composer) app.composer = null;
    else app.selection = [];
    return;
  }
  const combo = comboOf(e);
  if (!combo) return;
  const action = actionFor(combo);
  if (!action) return;
  if (action === 'settings') {
    e.preventDefault();
    settings.open = true;
    return;
  }
  if (!app.snap || (typing && !WHILE_TYPING.includes(action))) return;
  e.preventDefault();
  handlers[action]();
}
