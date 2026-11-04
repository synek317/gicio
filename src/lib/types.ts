export type Side = 'old' | 'new';
export type LineKind = 'context' | 'added' | 'removed';
export type FileStatus = 'added' | 'modified' | 'deleted' | 'renamed';
export type MarkStatus = 'accepted' | 'deferred';
export type ThreadStatus = 'open' | 'resolved' | 'deferred';

export interface DiffLine { kind: LineKind; oldNo: number | null; newNo: number | null; text: string }
export interface HunkPart { from: number; to: number }
export interface Hunk { id: string; path: string; oldStart: number; oldLen: number; newStart: number; newLen: number; lines: DiffLine[]; /** set on UI-side slices of a hunk */ baseId?: string }
export interface FileDiff { path: string; oldPath: string | null; status: FileStatus; binary: boolean; hunks: Hunk[] }
export interface HunkRef { id: string; path: string; oldStart: number; oldLen: number; part: HunkPart | null }
export type ChunkOrigin = { kind: 'fallback' } | { kind: 'ai' } | { kind: 'followUp'; thread: string | null };
export interface Note { path: string; side: Side; line: number; text: string; outdated?: boolean }
/** `group`: the feature the step belongs to; consecutive steps of one feature are shown nested. */
export interface Chunk { id: string; title: string; summary: string; hunks: HunkRef[]; origin: ChunkOrigin; notes: Note[]; group?: string | null }
export interface LineKey { path: string; side: Side; line: number }
export interface Mark extends LineKey { text: string; status: MarkStatus }
export interface AnchorInput { path: string; side: Side; start: number; end: number }
export interface Anchor extends AnchorInput { excerpt: string; outdated?: boolean }
export type Author = { kind: 'human' } | { kind: 'ai'; backend: string } | { kind: 'system' };
/** `change`: for a "code changed" note, the AI change it reports. */
export interface Message { author: Author; kind: 'text' | 'error' | 'codeChanged'; body: string; at: number; change?: string }
export type ChangeStatus = 'pending' | 'accepted' | 'rejected';
/** What one AI job (or tool) changed in the working tree, to look at and keep or undo. */
export interface AgentChange { id: string; title: string; by: string; thread: string | null; files: string[]; status: ChangeStatus }
export interface Thread { id: string; chunk: string | null; anchor: Anchor | null; status: ThreadStatus; allowEdits: boolean; messages: Message[]; busy: boolean }

export type BackendKind = 'claude' | 'codex' | 'copilot' | 'custom';
export type Engine = { kind: 'claude' | 'codex' | 'copilot' } | { kind: 'custom'; command: string };
export type Effort = 'low' | 'medium' | 'high' | 'xhigh' | 'max';
export interface AiBackend { engine: Engine; model: string | null; effort: Effort | null; language: string }
export interface InstructionFile { path: string; kind: string; skill: [string, string] | null; native: boolean }
export type JobKind = 'ask' | 'plan' | 'chat' | 'summary' | 'review' | 'aiReview' | 'fix' | 'commit' | 'tool';
export type JobStatus = 'queued' | 'running' | 'done' | 'failed' | 'cancelled';
export interface JobInfo { id: string; kind: JobKind; status: JobStatus; title: string; thread: string | null; log: string[] }
/** `ask`: the AI answers now; `later`: left for the "resolve later" step. */
export type SendMode = 'ask' | 'later';
export type StageMode = 'memory' | 'index';
/** What to do with the accepted changes once the review is finished. */
export type FinishAction = 'nothing' | 'stage' | 'commit' | 'amend';
export interface DiffOptions { ignoreWhitespace: boolean }
export type Severity = 'error' | 'warning' | 'info';
export type FindingStatus = 'open' | 'fixing' | 'fixed' | 'dismissed';
export interface Finding { id: string; source: 'ai' | 'clippy' | 'dotnet'; severity: Severity; path: string; side: Side; line: number; endLine: number; title: string; detail: string; status: FindingStatus; outdated?: boolean }
export type ReviewScope = { kind: 'all' } | { kind: 'chunk'; id: string } | { kind: 'lines'; path: string; side: Side; start: number; end: number };
export type Tool = 'fmt' | 'clippy' | 'clippyFix' | 'dotnetFormat' | 'dotnetBuild';
export interface Symbol { name: string; kind: number; start: number; end: number }

export interface Snapshot {
  version: number;
  repo: string;
  baseRef: string;
  baseCommit: string;
  backend: AiBackend;
  files: FileDiff[];
  chunks: Chunk[];
  marks: Mark[];
  threads: Thread[];
  chat: Message[];
  findings: Finding[];
  changes: AgentChange[];
  diffOptions: DiffOptions;
  stageMode: StageMode;
  summary: Message | null;
  chatBusy: boolean;
  summaryBusy: boolean;
  planning: boolean;
  plannedByAi: boolean;
  canUndo: boolean;
  notice: string | null;
  lastChunk: string | null;
  instructions: InstructionFile[];
}

/** How a missing language server gets installed; `available` when gicio can run the installer itself. */
export interface LspInstall { command: string; available: boolean; prerequisite: string }
/** The language server's state, which server it is (rust-analyzer, csharp-ls), the file extensions it
 *  understands and, when it is missing, how to install it. */
export type LspStatus = ({ state: 'off' | 'missing' | 'installing' | 'starting' | 'ready' } | { state: 'indexing' | 'failed'; detail: string }) & {
  server: string | null;
  extensions: string[];
  install: LspInstall | null;
};
export interface SourceLocation { absPath: string; repoPath: string | null; line: number; character: number; preview: string | null }
export interface LaunchArgs { repo: string | null; since: string | null; start: boolean; warning: string | null }
export interface FileTexts { old: string; new: string; binary: boolean }

/** A project search; `include` / `exclude` are comma-separated globs. */
export interface SearchQuery { pattern: string; caseSensitive: boolean; wholeWord: boolean; regex: boolean; include: string; exclude: string }
/** A line with matches. `line` is 1-based; `column` / `length` (UTF-16) are the first match in the whole
 *  line; `ranges` mark the matches inside `preview`, which is cut around them. */
export interface LineHit { line: number; column: number; length: number; preview: string; ranges: [number, number][]; matches: number }
export interface FileHits { path: string; absPath: string; lines: LineHit[] }
export interface SearchResults { files: FileHits[]; matches: number; truncated: boolean }
