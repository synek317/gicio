<script lang="ts">
  import { basicSetup } from 'codemirror';
  import { EditorState } from '@codemirror/state';
  import { EditorView, keymap } from '@codemirror/view';
  import { rust } from '@codemirror/lang-rust';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { currentChunkId } from '../actions';
  import { api } from '../api';
  import { lspExtensions } from '../cm-lsp';
  import { app } from '../state.svelte';

  let { absPath, repoPath, line, title }: { absPath: string; repoPath: string | null; line: number | null; title: string } = $props();

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

  function jumpTo(v: EditorView, n: number) {
    const l = v.state.doc.line(Math.max(1, Math.min(n, v.state.doc.lines)));
    v.dispatch({ selection: { anchor: l.from, head: l.to }, effects: EditorView.scrollIntoView(l.from, { y: 'center' }) });
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
              ...(absPath.endsWith('.rs') ? [rust()] : []),
              ...(dark ? [oneDark] : []),
              EditorState.readOnly.of(!path),
              keymap.of([{ key: 'Mod-s', preventDefault: true, run: () => (save(), true) }]),
              EditorView.updateListener.of((u) => u.docChanged && (app.editorDirty = true)),
              EditorView.theme({ '&': { height: '100%', fontSize: '12.5px' }, '.cm-scroller': { fontFamily: 'var(--mono)' } }),
              ...(path ? lspExtensions(path) : []),
            ],
          }),
        });
        view = ed;
        if (line) jumpTo(ed, line);
        ed.focus();
      },
      (e) => (error = String(e)),
    );
    return () => {
      ed?.destroy();
      view = null;
      app.editorDirty = false;
    };
  });

  $effect(() => {
    if (view && line) jumpTo(view, line);
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
