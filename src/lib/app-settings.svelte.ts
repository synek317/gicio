import { api, inTauri } from './api';
import type { BackendForm } from './backend-form';
import { defaultKeybindings, type Keybindings } from './keymap';

export interface Prefs {
  layout: 'split' | 'unified';
  order: 'ai' | 'files';
  /** After "Accept rest", jump to the next step with pending lines. */
  autoAdvance: boolean;
  notesExpanded: boolean;
  confirmStartOver: boolean;
  hoverDelayMs: number;
}

export interface AppSettings {
  defaultBackend: BackendForm;
  aiInstructions: string;
  prefs: Prefs;
  keybindings: Keybindings;
}

const LOCAL = 'gicio.settings';

export function defaultSettings(): AppSettings {
  return {
    defaultBackend: { kind: 'claude', command: '', model: '', effort: '', language: 'English' },
    aiInstructions: '',
    prefs: { layout: 'split', order: 'ai', autoAdvance: true, notesExpanded: false, confirmStartOver: true, hoverDelayMs: 450 },
    keybindings: defaultKeybindings(),
  };
}

function merge(raw: Partial<AppSettings> | null): AppSettings {
  const d = defaultSettings();
  if (!raw) return d;
  return {
    defaultBackend: { ...d.defaultBackend, ...raw.defaultBackend },
    aiInstructions: raw.aiInstructions ?? '',
    prefs: { ...d.prefs, ...raw.prefs },
    keybindings: { ...d.keybindings, ...raw.keybindings },
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
