import { hoverTooltip, EditorView, keymap } from '@codemirror/view';
import type { Extension } from '@codemirror/state';
import { api } from './api';
import { gotoDefinition, lspUsable } from './lsp';
import { renderMarkdown } from './markdown';
import { openMenu } from './menu';
import { app } from './state.svelte';
import { settings } from './app-settings.svelte';

/** rust-analyzer in CodeMirror: hover, ⌘-click definition, right-click menu. */
export function lspExtensions(path: string): Extension[] {
  const pos = (view: EditorView, p: number) => {
    const line = view.state.doc.lineAt(p);
    return { line: line.number, col: p - line.from };
  };
  return [
    hoverTooltip(async (view, p) => {
      if (!lspUsable(path) || app.editorDirty) return null;
      const { line, col } = pos(view, p);
      const text = await api.lspHover(path, line - 1, col).catch(() => null);
      if (!text) return null;
      return {
        pos: p,
        above: true,
        create: () => {
          const dom = document.createElement('div');
          dom.className = 'cm-lsp-hover md';
          dom.innerHTML = renderMarkdown(text);
          return { dom };
        },
      };
    }, { hoverTime: settings.value.prefs.hoverDelayMs }),
    keymap.of([
      {
        key: 'F2',
        run: (view) => {
          if (!lspUsable(path)) return false;
          const { line, col } = pos(view, view.state.selection.main.head);
          app.rename = { path, line, character: col };
          return true;
        },
      },
    ]),
    EditorView.domEventHandlers({
      mousedown(e, view) {
        if (!(e.metaKey || e.ctrlKey)) return false;
        const p = view.posAtCoords({ x: e.clientX, y: e.clientY });
        if (p == null) return false;
        e.preventDefault();
        const { line, col } = pos(view, p);
        gotoDefinition(path, line, col);
        return true;
      },
      contextmenu(e, view) {
        const p = view.posAtCoords({ x: e.clientX, y: e.clientY });
        if (p == null) return false;
        const { line, col } = pos(view, p);
        openMenu(e, path, 'new', line, col);
        return true;
      },
    }),
  ];
}
