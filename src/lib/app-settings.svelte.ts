import { api, inTauri } from './api';
import type { BackendForm } from './backend-form';
import { defaultKeybindings, type Keybindings } from './keymap';
import { systemLanguage } from './languages';
import { defaultStartOptions, type StartOptions } from './start-options';

export interface Prefs {
  layout: 'split' | 'unified';
  /** After "Accept rest", jump to the next step with pending lines. */
  autoAdvance: boolean;
  notesExpanded: boolean;
  confirmStartOver: boolean;
  hoverDelayMs: number;
  /** When a file has only blank lines left unaccepted. */
  blankLines: 'ask' | 'accept' | 'never';
  /** After a rename, lines that were already accepted. */
  renameAccepted: 'ask' | 'keep' | 'review';
}

export interface AppSettings {
  defaultBackend: BackendForm;
  /** Defaults for a new review; with `skipStartScreen` they are used without asking. */
  start: StartOptions;
  skipStartScreen: boolean;
  aiInstructions: string;
  /** Leave lock files and generated code out of AI planning, reviews and summaries. */
  skipLowValue: boolean;
  skipPatterns: string[];
  prefs: Prefs;
  keybindings: Keybindings;
}

const LOCAL = 'gicio.settings';

/** Mirrors `gicio_core::skip::DEFAULT_SKIP_PATTERNS`. */
export const DEFAULT_SKIP = [
  'Cargo.lock', 'package-lock.json', 'yarn.lock', 'pnpm-lock.yaml', 'bun.lockb', 'poetry.lock', 'uv.lock', 'Pipfile.lock',
  'Gemfile.lock', 'composer.lock', 'go.sum', '*.min.js', '*.min.css', '*.map', '*.snap', '*.pb.go', '*_generated.*',
  '*.generated.*', 'dist/**', 'vendor/**', 'node_modules/**',
];

export function defaultSettings(): AppSettings {
  return {
    defaultBackend: { kind: 'claude', command: '', model: '', effort: '', language: systemLanguage() },
    aiInstructions: '',
    skipLowValue: true,
    skipPatterns: [...DEFAULT_SKIP],
    start: defaultStartOptions(),
    skipStartScreen: false,
    prefs: { layout: 'split', autoAdvance: true, notesExpanded: false, confirmStartOver: true, hoverDelayMs: 450, blankLines: 'ask', renameAccepted: 'ask' },
    keybindings: defaultKeybindings(),
  };
}

/** Modifier + arrow combos are left to text editing (word and line jumps, selecting with Shift). */
const editingCombo = (c: string) => /arrow/.test(c) && /(mod|ctrl|alt|shift)\+/.test(c);

/** Saved shortcuts over the defaults. Actions that no longer exist are dropped; an action that lost
 *  editing combos gets the default ones back so it keeps at least two. */
function mergeKeybindings(defaults: Keybindings, saved: Partial<Keybindings> | undefined): Keybindings {
  const out = { ...defaults };
  for (const [id, combos] of Object.entries(saved ?? {}) as [keyof Keybindings, string[]][]) {
    if (!(id in defaults)) continue;
    const kept = combos.filter((c) => !editingCombo(c));
    out[id] = kept.length < combos.length ? [...new Set([...kept, ...defaults[id]])] : kept;
  }
  return out;
}

function merge(raw: Partial<AppSettings> | null): AppSettings {
  const d = defaultSettings();
  if (!raw) return d;
  // Older settings kept the review order among the preferences.
  const legacyOrder = (raw.prefs as { order?: 'ai' | 'files' } | undefined)?.order;
  return {
    defaultBackend: { ...d.defaultBackend, ...raw.defaultBackend },
    start: { ...d.start, ...(legacyOrder ? { order: legacyOrder } : {}), ...raw.start },
    skipStartScreen: raw.skipStartScreen ?? false,
    aiInstructions: raw.aiInstructions ?? '',
    skipLowValue: raw.skipLowValue ?? true,
    skipPatterns: raw.skipPatterns ?? [...DEFAULT_SKIP],
    prefs: { ...d.prefs, ...raw.prefs },
    keybindings: mergeKeybindings(d.keybindings, raw.keybindings),
  };
}

/** App-wide settings (settings window); stored by the backend next to the app config. */
class SettingsState {
  value = $state<AppSettings>(defaultSettings());
  open = $state(false);
  /** True while the shortcut editor waits for a key. */
  capturing = $state(false);

  async load() {
    try {
      const raw = inTauri ? await api.getAppSettings() : JSON.parse(localStorage.getItem(LOCAL) ?? 'null');
      this.value = merge(raw as Partial<AppSettings>);
    } catch {
      this.value = defaultSettings();
    }
  }

  async save(next: AppSettings) {
    this.value = next;
    if (inTauri) await api.saveAppSettings(next);
    else localStorage.setItem(LOCAL, JSON.stringify(next));
  }
}

export const settings = new SettingsState();
