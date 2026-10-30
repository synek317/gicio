import type { AiBackend, BackendKind, Effort } from './types';

export interface BackendForm { kind: BackendKind; command: string; model: string; effort: Effort | ''; language: string }

export function toBackend(f: BackendForm): AiBackend {
  return {
    engine: f.kind === 'custom' ? { kind: 'custom', command: f.command } : { kind: f.kind },
    model: f.kind === 'custom' ? null : f.model.trim() || null,
    effort: f.effort || null,
    language: f.language || 'English',
  };
}

export function fromBackend(b: AiBackend): BackendForm {
  return { kind: b.engine.kind, command: b.engine.kind === 'custom' ? b.engine.command : '', model: b.model ?? '', effort: b.effort ?? '', language: b.language };
}
