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
export interface Chunk { id: string; title: string; summary: string; hunks: HunkRef[]; origin: ChunkOrigin; notes: Note[] }
export interface LineKey { path: string; side: Side; line: number }
export interface Mark extends LineKey { text: string; status: MarkStatus }
export interface AnchorInput { path: string; side: Side; start: number; end: number }
export interface Anchor extends AnchorInput { excerpt: string; outdated?: boolean }
export type Author = { kind: 'human' } | { kind: 'ai'; backend: string } | { kind: 'system' };
export interface Message { author: Author; kind: 'text' | 'error' | 'codeChanged'; body: string; at: number; pending?: boolean }
export interface Thread { id: string; chunk: string | null; anchor: Anchor | null; status: ThreadStatus; allowEdits: boolean; todo: boolean; messages: Message[]; busy: boolean }

export type BackendKind = 'claude' | 'codex' | 'copilot' | 'custom';
export type Engine = { kind: 'claude' | 'codex' | 'copilot' } | { kind: 'custom'; command: string };
export type Effort = 'low' | 'medium' | 'high' | 'xhigh' | 'max';
export interface AiBackend { engine: Engine; model: string | null; effort: Effort | null; language: string }
export interface InstructionFile { path: string; kind: string; skill: [string, string] | null; native: boolean }
export type JobKind = 'ask' | 'todo' | 'plan' | 'chat' | 'summary' | 'review' | 'aiReview' | 'fix' | 'commit' | 'tool';
export type JobStatus = 'queued' | 'running' | 'done' | 'failed' | 'cancelled';
export interface JobInfo { id: string; kind: JobKind; status: JobStatus; title: string; thread: string | null; log: string[] }
export type SendMode = 'note' | 'ask' | 'todo' | 'review';
export type StageMode = 'memory' | 'index';
export interface DiffOptions { ignoreWhitespace: boolean }
export type Severity = 'error' | 'warning' | 'info';
export type FindingStatus = 'open' | 'fixing' | 'fixed' | 'dismissed';
export interface Finding { id: string; source: 'ai' | 'clippy'; severity: Severity; path: string; side: Side; line: number; endLine: number; title: string; detail: string; status: FindingStatus; outdated?: boolean }
export type ReviewScope = { kind: 'all' } | { kind: 'chunk'; id: string } | { kind: 'lines'; path: string; side: Side; start: number; end: number };
export type Tool = 'fmt' | 'clippy' | 'clippyFix';
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

export type LspStatus = { state: 'off' | 'starting' | 'ready' } | { state: 'indexing' | 'failed'; detail: string };
export interface SourceLocation { absPath: string; repoPath: string | null; line: number; character: number; preview: string | null }
export interface LaunchArgs { repo: string | null; since: string | null; start: boolean; warning: string | null }
export interface FileTexts { old: string; new: string; binary: boolean }
