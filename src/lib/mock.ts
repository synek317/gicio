// Browser-only stand-in for the Tauri backend, so the UI can be developed with `npm run dev`.
// Loads a snapshot dumped by `cargo run -p gicio-core --example cli -- <repo> <base> dump static/dev-fixture.json`.
import { compileFind, matchRanges } from './find-query';
import { keyOf } from './keys';
import type { FileHits, FileTexts, JobInfo, LineKey, LspStatus, Mark, MarkStatus, SearchQuery, SearchResults, Snapshot } from './types';

interface Fixture { snapshot: Snapshot; texts: Record<string, FileTexts>; jobs?: JobInfo[] }

let fixture: Fixture | null = null;
let undo: Mark[][] = [];
/** `?lsp=missing` (or `?lsp=noinstaller`) shows the prompt for a missing csharp-ls. */
const lspParam = typeof location === 'undefined' ? null : new URLSearchParams(location.search).get('lsp');
let mockLsp: LspStatus = lspParam
  ? { state: 'missing', server: 'csharp-ls', extensions: ['cs'], install: { command: 'dotnet tool update --global csharp-ls', available: lspParam !== 'noinstaller', prerequisite: 'the .NET SDK (https://dot.net)' } }
  : { state: 'off', server: null, extensions: [], install: null };

async function load(): Promise<Fixture> {
  fixture ??= await (await fetch('/dev-fixture.json')).json();
  return fixture!;
}

function lineText(s: Snapshot, k: LineKey): string | null {
  const f = s.files.find((f) => f.path === k.path);
  for (const h of f?.hunks ?? [])
    for (const l of h.lines) {
      if (k.side === 'new' && l.kind === 'added' && l.newNo === k.line) return l.text;
      if (k.side === 'old' && l.kind === 'removed' && l.oldNo === k.line) return l.text;
    }
  return null;
}

export async function mockInvoke(cmd: string, args: Record<string, any> = {}): Promise<unknown> {
  const fx = await load();
  const s = fx.snapshot;
  s.findings ??= [];
  s.diffOptions ??= { ignoreWhitespace: false };
  s.stageMode ??= 'memory';
  s.changes ??= [];
  const bump = () => ({ ...s, version: ++s.version, canUndo: undo.length > 0 });
  switch (cmd) {
    case 'open_session':
    case 'get_session':
    case 'refresh':
      return bump();
    case 'launch_args': {
      // Simulates `gicio PATH [--since REF] [--start]` with `?repo=…&since=…&start`.
      const q = new URLSearchParams(location.search);
      return q.get('repo') ? { repo: q.get('repo'), since: q.get('since'), start: q.has('start'), warning: null } : null;
    }
    case 'list_refs':
      return ['main', 'HEAD~1'];
    case 'file_texts':
      return fx.texts[args.path];
    case 'set_marks': {
      const map = new Map(s.marks.map((m) => [keyOf(m.path, m.side, m.line), m]));
      const prev: Mark[] = [...map.values()];
      for (const k of args.keys as LineKey[]) {
        const text = lineText(s, k);
        if (text == null) continue;
        if (args.status) map.set(keyOf(k.path, k.side, k.line), { ...k, text, status: args.status as MarkStatus });
        else map.delete(keyOf(k.path, k.side, k.line));
      }
      undo.push(prev);
      s.marks = [...map.values()];
      return bump();
    }
    case 'undo':
      s.marks = undo.pop() ?? s.marks;
      return bump();
    case 'create_thread': {
      const id = `t${s.threads.length + 100}`;
      s.threads.push({ id, chunk: args.chunk, anchor: args.anchor ? { ...args.anchor, excerpt: '' } : null, status: 'open', allowEdits: args.allowEdits, messages: [], busy: false });
      return id;
    }
    case 'send_message': {
      const t = s.threads.find((t) => t.id === args.thread)!;
      t.messages.push({ author: { kind: 'human' }, kind: 'text', body: args.body, at: Date.now() / 1000 });
      t.status = args.mode === 'later' ? 'deferred' : 'open';
      if (args.mode === 'ask') {
        t.messages.push({ author: { kind: 'ai', backend: 'mock' }, kind: 'text', body: 'Mock answer with `code` and\n\n```rust\nfn x() {}\n```', at: Date.now() / 1000 });
        // Pretend the AI edited the first file.
        const id = `a${s.changes.length + 1}`;
        const path = s.files[0].path;
        s.changes.push({ id, title: args.body.slice(0, 60), by: 'mock', thread: t.id, files: [path], status: 'pending' });
        t.messages.push({ author: { kind: 'system' }, kind: 'codeChanged', body: `Code changed in \`${path}\` (shown right after this step).`, at: Date.now() / 1000, change: id });
      }
      return bump();
    }
    case 'set_thread_status':
      s.threads.find((t) => t.id === args.thread)!.status = args.status;
      return bump();
    case 'plan_ai':
    case 'replan_ai':
      // Pretend the AI found two features, the first one in two steps.
      s.chunks.forEach((c, i) => (c.group = i < 2 ? 'Feature one' : i < 4 ? 'Feature two' : null));
      s.plannedByAi = true;
      return bump();
    case 'list_jobs':
      return fx.jobs ?? [];
    case 'cancel_job':
      return null;
    case 'chat_send':
      s.chat = [...(s.chat ?? []), { author: { kind: 'human' }, kind: 'text', body: args.body, at: Date.now() / 1000 }, { author: { kind: 'ai', backend: 'mock' }, kind: 'text', body: 'Mock chat answer.', at: Date.now() / 1000 }];
      return bump();
    case 'summarize':
      s.summary = { author: { kind: 'ai', backend: 'mock' }, kind: 'text', body: '**What it does**\n- Mock summary.', at: Date.now() / 1000 };
      bump();
      return null;
    case 'clear_chat':
      s.chat = [];
      return bump();
    case 'change_diff': {
      const f = s.files[0];
      const h = f.hunks[0];
      return [{ ...f, status: 'modified', hunks: [{ ...h, lines: [{ kind: 'removed', oldNo: 4, newNo: null, text: 'use std::fmt;' }, ...h.lines.slice(0, 6)] }] }];
    }
    case 'accept_change':
    case 'reject_change': {
      const c = s.changes.find((c) => c.id === args.id)!;
      c.status = cmd === 'accept_change' ? 'accepted' : 'rejected';
      return cmd === 'accept_change' ? bump() : [bump(), []];
    }
    case 'resolve_later':
      for (const t of s.threads.filter((t) => t.status === 'deferred')) {
        t.status = 'open';
        t.messages.push({ author: { kind: 'ai', backend: 'mock' }, kind: 'text', body: 'Mock: resolved.', at: Date.now() / 1000 });
      }
      return bump();
    case 'export_review':
      return '# Review (mock)';
    case 'ai_review':
    case 'run_tool':
      return null;
    case 'set_findings_status':
      s.findings = (s.findings ?? []).map((f) => (args.ids.includes(f.id) ? { ...f, status: args.status } : f));
      return bump();
    case 'fix_findings':
      s.findings = (s.findings ?? []).map((f) => (args.ids.includes(f.id) ? { ...f, status: 'fixing' } : f));
      return bump();
    case 'staged_stat':
      return ' src/a.rs | 3 ++-\n 1 file changed';
    case 'suggest_commit_message':
      return 'Add mock commit message\n\nBody.';
    case 'commit':
      return ['abc1234', bump()];
    case 'set_stage_mode':
      s.stageMode = args.mode;
      return bump();
    case 'set_last_chunk':
      return null;
    case 'plan_by_files':
      s.plannedByAi = false;
      return bump();
    case 'set_diff_options':
      s.diffOptions = args.options;
      return bump();
    case 'lsp_symbols':
      return [];
    case 'ai_models':
      return args.program === 'claude' ? ['fable', 'opus', 'sonnet'] : [];
    case 'ai_tools':
      return { claude: true, codex: false, copilot: false };
    case 'set_backend':
      s.backend = args.backend;
      return bump();
    case 'lsp_status':
      return mockLsp;
    case 'lsp_install':
      mockLsp = { ...mockLsp, state: 'installing', install: null };
      await new Promise((r) => setTimeout(r, 1500));
      mockLsp = { ...mockLsp, state: 'ready' };
      return null;
    case 'lsp_restart':
      mockLsp = { ...mockLsp, state: 'ready', install: null };
      return null;
    case 'reset_review':
      s.marks = [];
      s.threads = [];
      undo = [];
      return bump();
    case 'read_file': {
      const path = Object.keys(fx.texts).find((p) => String(args.absPath).endsWith('/' + p));
      if (!path) throw new Error('mock: file not in fixture');
      return fx.texts[path].new;
    }
    case 'save_file':
      fx.texts[args.path] = { ...fx.texts[args.path], new: args.text };
      return bump();
    case 'search_project':
      return mockSearch(fx, s.repo, args.query as SearchQuery);
    default:
      throw new Error(`mock: ${cmd} not supported`);
  }
}

/** Rough stand-in for the globs of a project search: `*.ext` by extension, anything else by substring. */
function globHit(list: string, path: string): boolean | null {
  const parts = list.split(',').map((p) => p.trim()).filter(Boolean);
  if (!parts.length) return null;
  return parts.some((p) => (p.startsWith('*.') ? path.endsWith(p.slice(1)) : path.includes(p.replace(/\*+/g, ''))));
}

/** Project search over the files of the fixture (new side). */
function mockSearch(fx: Fixture, repo: string, q: SearchQuery): SearchResults {
  const re = compileFind({ query: q.pattern, caseSensitive: q.caseSensitive, wholeWord: q.wholeWord, regex: q.regex });
  const files: FileHits[] = [];
  let matches = 0;
  for (const [path, t] of Object.entries(fx.texts).sort(([a], [b]) => a.localeCompare(b))) {
    if (globHit(q.include, path) === false || globHit(q.exclude, path) === true || t.binary) continue;
    const lines = t.new.split('\n').flatMap((text, i) => {
      const found = matchRanges(text, re, q.wholeWord);
      if (!found.length) return [];
      const indent = text.length - text.trimStart().length;
      const preview = text.trim();
      return [{ line: i + 1, column: found[0][0], length: found[0][1] - found[0][0], preview, ranges: found.map(([a, b]) => [a - indent, b - indent] as [number, number]), matches: found.length }];
    });
    if (!lines.length) continue;
    matches += lines.reduce((n, l) => n + l.matches, 0);
    files.push({ path, absPath: `${repo}/${path}`, lines });
  }
  return { files, matches, truncated: false };
}
