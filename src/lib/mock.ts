// Browser-only stand-in for the Tauri backend, so the UI can be developed with `npm run dev`.
// Loads a snapshot dumped by `cargo run -p gicio-core --example cli -- <repo> <base> dump static/dev-fixture.json`.
import { keyOf } from './keys';
import type { FileTexts, JobInfo, LineKey, Mark, MarkStatus, Snapshot } from './types';

interface Fixture { snapshot: Snapshot; texts: Record<string, FileTexts>; jobs?: JobInfo[] }

let fixture: Fixture | null = null;
let undo: Mark[][] = [];

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
  const bump = () => ({ ...s, version: ++s.version, canUndo: undo.length > 0 });
  switch (cmd) {
    case 'open_session':
    case 'get_session':
    case 'refresh':
      return bump();
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
      s.threads.push({ id, chunk: args.chunk, anchor: args.anchor ? { ...args.anchor, excerpt: '' } : null, status: 'open', allowEdits: args.allowEdits, todo: args.todo, messages: [], busy: false });
      return id;
    }
    case 'send_message': {
      const t = s.threads.find((t) => t.id === args.thread)!;
      t.messages.push({ author: { kind: 'human' }, kind: 'text', body: args.body, at: Date.now() / 1000 });
      if (args.mode !== 'note') t.messages.push({ author: { kind: 'ai', backend: 'mock' }, kind: 'text', body: 'Mock answer with `code` and\n\n```rust\nfn x() {}\n```', at: Date.now() / 1000 });
      return bump();
    }
    case 'set_thread_status':
      s.threads.find((t) => t.id === args.thread)!.status = args.status;
      return bump();
    case 'plan_ai':
    case 'replan_ai':
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
    case 'set_backend':
      s.backend = args.backend;
      return bump();
    case 'lsp_status':
      return { state: 'off' };
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
    default:
      throw new Error(`mock: ${cmd} not supported`);
  }
}
