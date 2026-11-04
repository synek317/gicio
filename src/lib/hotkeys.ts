import { api } from './api';
import { acceptRestAndNext, back, closeEditor, editFocused, focusChat, forward, go, markTarget, moveChunk, moveFocus, nextPendingChunk, openComposer, run } from './actions';
import { settings } from './app-settings.svelte';
import { jumpFile, toggleCurrentFile } from './files-nav';
import { closeFind, find, findStep, openFind } from './find.svelte';
import { comboOf, type ActionId } from './keymap';
import { togglePanel } from './panels.svelte';
import { openProjectSearch } from './project-search.svelte';
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
  rename: () => {
    if (app.pointerCode) app.rename = { ...app.pointerCode };
    else app.toast('Point at a symbol (language server running) and press F2, or right-click it.');
  },
  edit: () => editFocused(),
  toggleLayout: () => (app.layout = app.layout === 'split' ? 'unified' : 'split'),
  laterView: () => go({ kind: 'later' }),
  undo: () => run(api.undo()),
  reload: () => run(api.refresh()),
  back: () => back(),
  forward: () => forward(),
  chat: () => focusChat(),
  find: () => openFind(),
  findNext: () => findStep(1),
  findPrev: () => findStep(-1),
  searchProject: () => openProjectSearch(),
  toggleLeftPanel: () => togglePanel('left'),
  toggleRightPanel: () => togglePanel('right'),
  settings: () => (settings.open = true),
  help: () => (app.showHelp = !app.showHelp),
};

/** Actions that also work while typing (in the editor or a text box). */
const WHILE_TYPING: ActionId[] = ['back', 'forward', 'settings', 'find', 'findNext', 'findPrev', 'searchProject'];
/** Actions a code editor hands over to the app instead of running its own (its search panel). */
const FROM_EDITOR: ActionId[] = ['find', 'findNext', 'findPrev', 'searchProject'];

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
    else if (find.open && !typing) closeFind();
    else if (app.composer) app.composer = null;
    else if (app.editing) closeEditor();
    else app.selection = [];
    return;
  }
  const combo = comboOf(e);
  if (!combo) return;
  // While typing, arrows (with any modifier) move the cursor, and keys a text field or the editor
  // already handled (e.g. ⌘[ outdents in the editor) are theirs.
  if (typing && (combo.includes('arrow') || e.defaultPrevented)) return;
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

/** Key handler for code editors: runs the find shortcuts before the editor's own key bindings. */
export function editorFindKey(e: KeyboardEvent): boolean {
  const combo = comboOf(e);
  const action = combo && actionFor(combo);
  if (!action || !FROM_EDITOR.includes(action)) return false;
  e.preventDefault();
  handlers[action]();
  return true;
}
