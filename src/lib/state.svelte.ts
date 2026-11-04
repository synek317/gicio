import { changedKeys, keyOf } from './keys';
import { hunksProgress, type Progress } from './progress';
import { sliceHunk } from './slice';
import type { Finding, AnchorInput, Chunk, FileDiff, Hunk, JobInfo, LspStatus, MarkStatus, Note, Snapshot, SourceLocation, Thread } from './types';

export type View =
  | { kind: 'chunk'; id: string }
  | { kind: 'later' }
  | { kind: 'file'; path: string }
  /** `col` / `len`: a range of `line` to select (UTF-16), e.g. a search match; else the whole line. */
  | { kind: 'source'; title: string; absPath: string; repoPath: string | null; line: number | null; col?: number; len?: number }
  | { kind: 'refs'; title: string; locs: SourceLocation[] }
  | { kind: 'change'; id: string };
export interface HistoryEntry { view: View; scroll: number }
export interface Toast { id: number; text: string; kind: 'info' | 'error' }
export interface HoverInfo { x: number; y: number; text: string }
export interface ContextMenu { x: number; y: number; path: string; side: 'old' | 'new'; line: number; character: number }

/** All UI state; the backend snapshot is the source of truth for review data. */
class AppState {
  snap = $state<Snapshot | null>(null);
  view = $state<View | null>(null);
  history = $state<HistoryEntry[]>([]);
  future = $state<HistoryEntry[]>([]);
  layout = $state<'split' | 'unified'>('split');
  selection = $state<string[]>([]);
  selAnchor = $state<string | null>(null);
  /** The line the selection was last extended to, where the pointer is; the popover sits there. */
  selHead = $state<string | null>(null);
  focusedHunk = $state<string | null>(null);
  /** Text to prefill the next composer with (e.g. from a finding). */
  composerDraft = $state('');
  /** A new conversation about lines or a step (the whole change has its own card). */
  composer = $state<{ anchor: AnchorInput | null; chunk: string | null } | null>(null);
  activeThread = $state<string | null>(null);
  /** Bumped to open the whole-change conversation and put the cursor in it. */
  chatFocus = $state(0);
  /** Which conversation history the panel shows: what is on screen (the step) or the whole change. */
  convScope = $state<'here' | 'whole'>('here');
  /** Bumped to put the cursor in the question box next to the selection. */
  selectionFocus = $state(0);
  /** Hunk (view id) being edited in place; the editor opens at `line` (new side), which was `offset`
   *  pixels below the top of the diff scroller, so the code stays where it was on screen. */
  editing = $state<{ hunk: string; line: number | null; offset: number | null } | null>(null);
  sidebarTab = $state<'steps' | 'files' | 'search'>('steps');
  rightTab = $state<'conversations' | 'findings'>('conversations');
  showHelp = $state(false);
  lsp = $state<LspStatus>({ state: 'off', server: null, extensions: [], install: null });
  /** The language server prompt: shown by itself when the server is missing (`auto`), opened from the
   *  status bar (`open`), or put away for this session (`closed`). */
  lspPrompt = $state<'auto' | 'open' | 'closed'>('auto');
  toasts = $state<Toast[]>([]);
  hover = $state<HoverInfo | null>(null);
  menu = $state<ContextMenu | null>(null);
  editorDirty = $state(false);
  /** Open rename dialog: where the symbol is (1-based line, UTF-16 column). */
  rename = $state<{ path: string; line: number; character: number } | null>(null);
  /** Bumped when files change under an open file view, so it reloads. */
  fileRev = $state(0);
  /** Code position under the pointer (for F2 rename in the diff). */
  pointerCode: { path: string; line: number; character: number } | null = null;
  /** First key of the block of changed lines whose accept bar is under the pointer. */
  hoverBlock = $state<string | null>(null);
  /** Lines an accept button under the pointer would accept. */
  preview = $state<Set<string>>(new Set());
  /** A file where only blank lines are left unaccepted, waiting for the user's answer. */
  blankPrompt = $state<{ path: string; keys: string[] } | null>(null);
  /** Extra context lines shown around hunks (by hunk view id). */
  expand = $state<Record<string, { before: number; after: number }>>({});
  /** Collapsed file blocks, by `${view id}|${path}`. */
  collapsed = $state<Record<string, boolean>>({});
  jobs = $state<JobInfo[]>([]);
  /** Files whose diff blocks are on screen (path -> count) and the file under the pointer. */
  onScreen = $state<Record<string, number>>({});
  hoverPath = $state<string | null>(null);

  marks = $derived(new Map<string, MarkStatus>((this.snap?.marks ?? []).map((m) => [keyOf(m.path, m.side, m.line), m.status])));
  hunks = $derived(new Map<string, Hunk>((this.snap?.files ?? []).flatMap((f) => f.hunks.map((h) => [h.id, h] as const))));
  files = $derived(new Map<string, FileDiff>((this.snap?.files ?? []).map((f) => [f.path, f])));
  selSet = $derived(new Set(this.selection));

  viewHunks = $derived.by((): Hunk[] => {
    const v = this.view;
    if (!v || !this.snap) return [];
    if (v.kind === 'chunk') {
      const c = this.chunk(v.id);
      return c ? this.chunkHunks(c) : [];
    }
    if (v.kind === 'file') return this.files.get(v.path)?.hunks ?? [];
    if (v.kind === 'later') return this.snap.files.flatMap((f) => f.hunks).filter((h) => changedKeys(h).some((k) => this.marks.get(k) === 'deferred'));
    return [];
  });
  viewHunkMap = $derived(new Map(this.viewHunks.map((h) => [h.id, h])));

  /** Changed lines of the current view in reading order; drives range selection. */
  orderedKeys = $derived(this.viewHunks.flatMap(changedKeys));

  overall = $derived(hunksProgress(this.snap?.files.flatMap((f) => f.hunks) ?? [], this.marks));

  /** `${path}|${side}|${line}` -> threads anchored there, for gutter markers. */
  threadLines = $derived.by(() => {
    const m = new Map<string, Thread[]>();
    for (const t of this.snap?.threads ?? []) {
      const a = t.anchor;
      // A conversation whose code is gone, or that is resolved, has no place in the code any more.
      if (!a || a.outdated || t.status === 'resolved') continue;
      for (let l = a.start; l <= a.end; l++) {
        const k = `${a.path}|${a.side}|${l}`;
        m.set(k, [...(m.get(k) ?? []), t]);
      }
    }
    return m;
  });

  /** `${path}|${side}|${line}` -> open findings covering that line. */
  findingLines = $derived.by(() => {
    const m = new Map<string, Finding[]>();
    for (const f of this.snap?.findings ?? []) {
      if ((f.status !== 'open' && f.status !== 'fixing') || f.outdated) continue;
      for (let l = f.line; l <= Math.max(f.line, f.endLine); l++) {
        const k = `${f.path}|${f.side}|${l}`;
        m.set(k, [...(m.get(k) ?? []), f]);
      }
    }
    return m;
  });

  /** The step being reviewed: the one on screen, else the last one visited (e.g. while a file is open). */
  stepId = $derived.by((): string | null => {
    const v = this.view;
    if (v?.kind === 'chunk') return v.id;
    for (const h of this.history.toReversed()) if (h.view.kind === 'chunk') return h.view.id;
    return null;
  });
  /** Files of that step. */
  stepPaths = $derived.by(() => {
    const c = this.stepId ? this.chunk(this.stepId) : undefined;
    return new Set(c ? this.chunkHunks(c).map((h) => h.path) : []);
  });

  /** The topmost file of the view that is on screen; the file tree follows it. */
  currentFile = $derived(this.viewHunks.map((h) => h.path).find((p) => this.onScreen[p]) ?? null);

  /** AI changes waiting for the reviewer to keep or undo them. */
  pendingChanges = $derived((this.snap?.changes ?? []).filter((c) => c.status === 'pending'));
  /** Conversations left for the "resolve later" step. */
  laterThreads = $derived((this.snap?.threads ?? []).filter((t) => t.status === 'deferred'));
  /** Anything put off: lines or conversations. */
  hasLater = $derived(this.overall.deferred > 0 || this.laterThreads.length > 0);
  /** Every changed line accepted, nothing left for later and the AI idle: the review is done. */
  finished = $derived(
    !!this.snap && this.overall.total > 0 && this.overall.pending === 0 && !this.hasLater && !this.snap.planning
      && this.pendingChanges.length === 0 && !this.snap.threads.some((t) => t.busy),
  );
  /** The "review finished" dialog. */
  showFinish = $state(false);

  /** `${path}|${side}|${line}` -> AI notes from the plan. */
  noteLines = $derived.by(() => {
    const m = new Map<string, Note[]>();
    for (const c of this.snap?.chunks ?? [])
      for (const n of (c.notes ?? []).filter((n) => !n.outdated)) {
        const k = `${n.path}|${n.side}|${n.line}`;
        m.set(k, [...(m.get(k) ?? []), n]);
      }
    return m;
  });

  chunk(id: string): Chunk | undefined {
    return this.snap?.chunks.find((c) => c.id === id);
  }

  chunkHunks(c: Chunk): Hunk[] {
    return c.hunks.flatMap((r) => {
      const h = this.hunks.get(r.id);
      return h ? [sliceHunk(h, r)] : [];
    });
  }

  chunkProgress(c: Chunk): Progress {
    return hunksProgress(this.chunkHunks(c), this.marks);
  }

  applySnapshot(s: Snapshot) {
    const cur = this.snap;
    if (cur && cur.baseCommit === s.baseCommit && cur.repo === s.repo && s.version < cur.version) return;
    this.snap = s;
    if (this.view?.kind === 'chunk' && !this.chunk(this.view.id)) this.view = s.chunks[0] ? { kind: 'chunk', id: s.chunks[0].id } : null;
    if (!this.view && s.chunks[0]) this.view = { kind: 'chunk', id: s.chunks[0].id };
    if (this.focusedHunk && !this.viewHunkMap.has(this.focusedHunk)) this.focusedHunk = null;
    // The code under an in-place editor changed (an edit elsewhere, the AI): its region is gone.
    if (this.editing && !this.viewHunkMap.has(this.editing.hunk)) {
      this.editing = null;
      this.editorDirty = false;
    }
    const live = new Set(this.orderedKeys);
    if (this.selection.some((k) => !live.has(k))) this.selection = this.selection.filter((k) => live.has(k));
  }

  jobLog(id: string, line: string) {
    this.jobs = this.jobs.map((j) => (j.id === id ? { ...j, log: [...j.log.slice(-399), line] } : j));
  }

  jobForThread(thread: string): JobInfo | undefined {
    return this.jobs.findLast((j) => j.thread === thread && (j.status === 'running' || j.status === 'queued'));
  }

  toast(text: string, kind: Toast['kind'] = 'info', ms?: number) {
    const id = Date.now() + Math.random();
    this.toasts = [...this.toasts, { id, text, kind }];
    setTimeout(() => (this.toasts = this.toasts.filter((t) => t.id !== id)), ms ?? (kind === 'error' ? 9000 : 3500));
  }
}

export const app = new AppState();
