import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { SearchQuery, SearchResults, LaunchArgs, DiffOptions, FileDiff, FindingStatus, ReviewScope, StageMode, Symbol, Tool, AiBackend, AnchorInput, FileTexts, JobInfo, LineKey, LspStatus, MarkStatus, SendMode, Snapshot, SourceLocation, ThreadStatus } from './types';

export const inTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!inTauri) return (await import('./mock')).mockInvoke(cmd, args) as Promise<T>;
  return invoke<T>(cmd, args);
}

export const api = {
  openSession: (repo: string, base: string, backend: AiBackend, stageMode: StageMode) => call<Snapshot>('open_session', { repo, base, backend, stageMode }),
  getSession: () => call<Snapshot>('get_session'),
  launchArgs: () => call<LaunchArgs | null>('launch_args'),
  refresh: () => call<Snapshot>('refresh'),
  listRefs: (repo: string) => call<string[]>('list_refs', { repo }),
  fileTexts: (path: string) => call<FileTexts>('file_texts', { path }),
  readFile: (absPath: string) => call<string>('read_file', { absPath }),
  setMarks: (keys: LineKey[], status: MarkStatus | null) => call<Snapshot>('set_marks', { keys, status }),
  undo: () => call<Snapshot>('undo'),
  resetReview: () => call<Snapshot>('reset_review'),
  saveFile: (path: string, text: string, after: string | null) => call<Snapshot>('save_file', { path, text, after }),
  planAi: () => call<Snapshot>('plan_ai'),
  replanAi: () => call<Snapshot>('replan_ai'),
  createThread: (chunk: string | null, anchor: AnchorInput | null, allowEdits: boolean) => call<string>('create_thread', { chunk, anchor, allowEdits }),
  sendMessage: (thread: string, body: string, mode: SendMode, allowEdits: boolean | null) => call<Snapshot>('send_message', { thread, body, mode, allowEdits }),
  listJobs: () => call<JobInfo[]>('list_jobs'),
  cancelJob: (id: string) => call<void>('cancel_job', { id }),
  setBackend: (backend: AiBackend) => call<Snapshot>('set_backend', { backend }),
  aiTools: () => call<Record<string, boolean>>('ai_tools'),
  aiModels: (program: string) => call<string[]>('ai_models', { program }),
  getAppSettings: () => call<unknown>('get_app_settings'),
  saveAppSettings: (settings: unknown) => call<void>('save_app_settings', { settings }),
  chatSend: (body: string, allowEdits: boolean) => call<Snapshot>('chat_send', { body, allowEdits }),
  summarize: () => call<void>('summarize'),
  clearChat: () => call<Snapshot>('clear_chat'),
  resolveLater: () => call<Snapshot>('resolve_later'),
  exportReview: () => call<string>('export_review'),
  aiReview: (scope: ReviewScope, focus: string | null) => call<void>('ai_review', { scope, focus }),
  setFindingsStatus: (ids: string[], status: FindingStatus) => call<Snapshot>('set_findings_status', { ids, status }),
  fixFindings: (ids: string[]) => call<Snapshot>('fix_findings', { ids }),
  stagedStat: () => call<string>('staged_stat'),
  suggestCommitMessage: () => call<string>('suggest_commit_message'),
  commit: (message: string) => call<[string, Snapshot]>('commit', { message }),
  runTool: (tool: Tool) => call<string>('run_tool', { tool }),
  setDiffOptions: (options: DiffOptions) => call<Snapshot>('set_diff_options', { options }),
  setStageMode: (mode: StageMode) => call<Snapshot>('set_stage_mode', { mode }),
  planByFiles: () => call<Snapshot>('plan_by_files'),
  setLastChunk: (chunk: string | null) => call<void>('set_last_chunk', { chunk }),
  writeTextFile: (path: string, text: string) => call<void>('write_text_file', { path, text }),
  lspPrepareRename: (path: string, line: number, character: number) => call<string | null>('lsp_prepare_rename', { path, line, character }),
  lspRename: (path: string, line: number, character: number, newName: string, keepAccepted: boolean, after: string | null) =>
    call<{ files: number; edits: number }>('lsp_rename', { path, line, character, newName, keepAccepted, after }),
  lspSymbols: (path: string) => call<Symbol[]>('lsp_symbols', { path }),
  changeDiff: (id: string) => call<FileDiff[]>('change_diff', { id }),
  acceptChange: (id: string) => call<Snapshot>('accept_change', { id }),
  rejectChange: (id: string) => call<[Snapshot, string[]]>('reject_change', { id }),
  setThreadStatus: (thread: string, status: ThreadStatus) => call<Snapshot>('set_thread_status', { thread, status }),
  lspStatus: () => call<LspStatus>('lsp_status'),
  lspInstall: () => call<null>('lsp_install'),
  lspRestart: () => call<null>('lsp_restart'),
  lspHover: (path: string, line: number, character: number) => call<string | null>('lsp_hover', { path, line, character }),
  lspDefinition: (path: string, line: number, character: number) => call<SourceLocation[]>('lsp_definition', { path, line, character }),
  lspReferences: (path: string, line: number, character: number) => call<SourceLocation[]>('lsp_references', { path, line, character }),
  searchProject: (query: SearchQuery) => call<SearchResults>('search_project', { query }),
};

export function onSession(cb: (s: Snapshot) => void): () => void {
  if (!inTauri) return () => {};
  let unlisten: (() => void) | null = null;
  listen<Snapshot>('session', (e) => cb(e.payload)).then((u) => (unlisten = u));
  return () => unlisten?.();
}

export function onJobs(cb: (jobs: JobInfo[]) => void, onLog: (id: string, line: string) => void): () => void {
  if (!inTauri) return () => {};
  const offs: (() => void)[] = [];
  listen<JobInfo[]>('jobs', (e) => cb(e.payload)).then((u) => offs.push(u));
  listen<{ id: string; line: string }>('job-log', (e) => onLog(e.payload.id, e.payload.line)).then((u) => offs.push(u));
  return () => offs.forEach((f) => f());
}

/** Files changed on disk outside gicio and the diff was re-read. */
export function onWorktreeChanged(cb: (files: string[]) => void): () => void {
  if (!inTauri) return () => {};
  let unlisten: (() => void) | null = null;
  listen<{ files: string[] }>('worktree-changed', (e) => cb(e.payload.files)).then((u) => (unlisten = u));
  return () => unlisten?.();
}
