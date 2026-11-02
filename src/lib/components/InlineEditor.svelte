<script lang="ts">
  import { untrack } from 'svelte';
  import { minimalSetup } from 'codemirror';
  import { EditorState } from '@codemirror/state';
  import { EditorView, highlightActiveLine, keymap, lineNumbers } from '@codemirror/view';
  import { unifiedMergeView } from '@codemirror/merge';
  import { rust } from '@codemirror/lang-rust';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { currentChunkId } from '../actions';
  import { api } from '../api';
  import { app } from '../state.svelte';
  import type { Hunk } from '../types';

  /** Edits the lines a hunk shows (its new side plus any extra context) right in the diff. It shows
   *  the same diff while you type: removed lines inline, added and changed text highlighted.
   *  The region is fixed when editing starts; snapshots arriving meanwhile do not move it. */
  let { hunk }: { hunk: Hunk } = $props();

  const { path, from, to, original } = untrack(() => {
    const news = hunk.lines.map((l) => l.newNo).filter((n): n is number => n != null);
    return {
      path: hunk.path,
      // A hunk that only removes lines has none on the new side: the edit inserts after `newStart`.
      from: news[0] ?? hunk.newStart + 1,
      to: news.at(-1) ?? hunk.newStart,
      original: hunk.lines.filter((l) => l.kind !== 'added').map((l) => l.text).join('\n'),
    };
  });
  const absPath = `${app.snap!.repo}/${path}`;

  let host: HTMLElement | undefined = $state();
  let view = $state.raw<EditorView | null>(null);
  let onDisk = '';
  let saving = $state(false);

  const regionOf = (text: string) => text.split('\n').slice(from - 1, to).join('\n');

  async function save() {
    if (!view || saving) return;
    saving = true;
    try {
      const text = await api.readFile(absPath);
      if (regionOf(text) !== onDisk) throw new Error('The file changed on disk meanwhile. Close the editor and edit again.');
      const lines = text.split('\n');
      const edited = view.state.doc.toString();
      const next = [...lines.slice(0, from - 1), ...(edited === '' ? [] : edited.split('\n')), ...lines.slice(to)].join('\n');
      app.applySnapshot(await api.saveFile(path, next, currentChunkId()));
      app.editorDirty = false;
      app.editing = null;
      app.toast('Saved. Lines you wrote count as accepted.');
    } catch (e) {
      app.toast(String(e), 'error');
    } finally {
      saving = false;
    }
  }

  /** Cursor on the line the reviewer was at, kept where it was on screen (else centred). */
  function jump(ed: EditorView) {
    const target = untrack(() => app.editing);
    const doc = ed.state.doc;
    const at = doc.line(Math.min(doc.lines, Math.max(1, (target?.line ?? from) - from + 1))).from;
    ed.contentDOM.focus({ preventScroll: true });
    ed.dispatch({ selection: { anchor: at } });
    // Measured after layout: the editor replaced the diff rows and may be taller or shorter.
    requestAnimationFrame(() => {
      const scroller = document.getElementById('diff-scroll');
      const line = ed.coordsAtPos(at);
      if (scroller && line && target?.offset != null) scroller.scrollTop += line.top - (scroller.getBoundingClientRect().top + target.offset);
      else ed.dispatch({ effects: EditorView.scrollIntoView(at, { y: 'center' }) });
    });
  }

  $effect(() => {
    if (!host) return;
    const el = host;
    let ed: EditorView | null = null;
    api.readFile(absPath).then(
      (text) => {
        onDisk = regionOf(text);
        const dark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        ed = new EditorView({
          parent: el,
          state: EditorState.create({
            doc: onDisk,
            extensions: [
              minimalSetup,
              lineNumbers({ formatNumber: (n) => String(n + from - 1) }),
              highlightActiveLine(),
              // Like the diff view: whole added and removed lines, no character-level marks.
              unifiedMergeView({ original, mergeControls: false, gutter: false, highlightChanges: false }),
              EditorView.lineWrapping,
              ...(path.endsWith('.rs') ? [rust()] : []),
              ...(dark ? [oneDark] : []),
              keymap.of([
                { key: 'Mod-s', preventDefault: true, run: () => (save(), true) },
                { key: 'Mod-Enter', preventDefault: true, run: () => (save(), true) },
              ]),
              EditorView.updateListener.of((u) => u.docChanged && (app.editorDirty = true)),
              EditorView.theme({ '&': { fontSize: '12.5px' }, '.cm-scroller': { fontFamily: 'var(--mono)', lineHeight: '1.55' } }),
            ],
          }),
        });
        view = ed;
        jump(ed);
      },
      (e) => {
        app.toast(String(e), 'error');
        app.editing = null;
      },
    );
    return () => {
      ed?.destroy();
      view = null;
    };
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="inline-editor" onclick={(e) => e.stopPropagation()}>
  <div class="inline-editor-host" bind:this={host}></div>
  <div class="inline-editor-bar row">
    <span class="muted small">Editing lines {from}–{Math.max(from, to)}{#if app.editorDirty} · ● unsaved{/if} · what you write counts as accepted · <kbd>Esc</kbd> closes</span>
    <span class="spacer"></span>
    <button class="small primary" disabled={saving || !app.editorDirty} onclick={save}>{saving ? 'Saving…' : 'Save'} <kbd>⌘S</kbd></button>
  </div>
</div>
