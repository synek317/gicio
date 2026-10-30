import type { BackendKind, Effort } from './types';

/** Start-screen choices, remembered between runs. */
export interface Settings {
  repo: string;
  scope: 'uncommitted' | 'since';
  stage: 'memory' | 'index';
  base: string;
  kind: BackendKind;
  model: string;
  effort: Effort | '';
  command: string;
  language: string;
  order: 'ai' | 'files';
  recent: string[];
}

const KEY = 'gicio.start';
const DEFAULTS: Settings = { repo: '', scope: 'uncommitted', stage: 'memory', base: 'main', kind: 'claude', model: '', effort: '', command: '', language: 'English', order: 'ai', recent: [] };

export function loadSettings(): Settings {
  try {
    return { ...DEFAULTS, ...JSON.parse(localStorage.getItem(KEY) ?? '{}') };
  } catch {
    return { ...DEFAULTS };
  }
}

export function saveSettings(s: Settings) {
  try {
    localStorage.setItem(KEY, JSON.stringify(s));
  } catch {
    /* private mode */
  }
}
