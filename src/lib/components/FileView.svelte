<script lang="ts">
  import { basicSetup } from 'codemirror';
  import { EditorState } from '@codemirror/state';
  import { EditorView, keymap } from '@codemirror/view';
  import { languageFor } from '../cm-language';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { currentChunkId } from '../actions';
  import { api } from '../api';
  import { lspExtensions } from '../cm-lsp';
  import { cmFindTarget, findExtensions } from '../cm-find';
  import { setFindTarget } from '../find.svelte';
  import { app } from '../state.svelte';

  let { absPath, repoPath, line, col = null, len = 0, title }: { absPath: string; repoPath: string | null; line: number | null; col?: number | null; len?: number; title: string } = $props();

  let host: HTMLElement | undefined = $state();
  let view = $state.raw<EditorView | null>(null);
  let saving = $state(false);
  let error = $state('');
  const editable = $derived(!!repoPath);

  async function save() {
    if (!view || !repoPath || saving) return;
    saving = true;
    try {
      app.applySnapshot(await api.saveFile(repoPath, view.state.doc.toString(), currentChunkId()));
      app.editorDirty = false;
      app.toast('Saved. Lines you wrote count as accepted.');
    } catch (e) {
      app.toast(String(e), 'error');
    } finally {
      saving = false;
    }
  }

  /** Selects line `n`, or `length` characters of it from `column` (a search match). */
  function jumpTo(v: EditorView, n: number, column: number | null, length: number) {
    const l = v.state.doc.line(Math.max(1, Math.min(n, v.state.doc.lines)));
    const from = column != null && length ? Math.min(l.from + column, l.to) : l.from;
    const to = column != null && length ? Math.min(from + length, l.to) : l.to;
    v.dispatch({ selection: { anchor: from, head: to }, effects: EditorView.scrollIntoView(from, { y: 'center' }) });
  }

  // One editor per file; going to another line of the same file only moves the cursor.
  $effect(() => {
    if (!host) return;
    const el = host;
    const path = repoPath;
    let ed: EditorView | null = null;
    api.readFile(absPath).then(
      (text) => {
        const dark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        ed = new EditorView({
          parent: el,
          state: EditorState.create({
            doc: text,
            extensions: [
              basicSetup,
              ...languageFor(absPath),
              ...(dark ? [oneDark] : []),
              EditorState.readOnly.of(!path),
              keymap.of([{ key: 'Mod-s', preventDefault: true, run: () => (save(), true) }]),
              EditorView.updateListener.of((u) => u.docChanged && (app.editorDirty = true)),
              EditorView.theme({ '&': { height: '100%', fontSize: '12.5px' }, '.cm-scroller': { fontFamily: 'var(--mono)' } }),
              ...(path ? lspExtensions(path) : []),
              findExtensions,
            ],
          }),
        });
        view = ed;
        if (line) jumpTo(ed, line, col, len);
        setFindTarget(cmFindTarget(ed));
        ed.focus();
      },
      (e) => (error = String(e)),
    );
    return () => {
      if (ed) setFindTarget(null);
      ed?.destroy();
      view = null;
      app.editorDirty = false;
    };
  });

  $effect(() => {
    if (view && line) jumpTo(view, line, col, len);
  });
</script>

<div class="editor-view">
  <header class="chunk-head row">
    <h1 class="mono">{title}</h1>
    {#if !editable}<span class="badge">outside repo · read only</span>{/if}
    {#if app.editorDirty}<span class="muted">● unsaved</span>{/if}
    <span class="spacer"></span>
    <span class="muted small">⌘-click: definition · right-click: references · ⌘− back</span>
    {#if editable}<button class="primary" disabled={saving || !app.editorDirty} onclick={save}>{saving ? 'Saving…' : 'Save'} <kbd>⌘S</kbd></button>{/if}
  </header>
  {#if error}<p class="error">{error}</p>{/if}
  <div class="editor-host" bind:this={host}></div>
</div>
