/** Rebindable actions. Every action ships with at least two key combos. None uses a modifier with an
 *  arrow key: those move the cursor by word or line (and select with Shift) while typing. */
export const ACTIONS = [
  { id: 'nextHunk', label: 'Next hunk', keys: ['j', 'shift+j'] },
  { id: 'prevHunk', label: 'Previous hunk', keys: ['k', 'shift+k'] },
  { id: 'nextChunk', label: 'Next step', keys: ['n', '.'] },
  { id: 'prevChunk', label: 'Previous step', keys: ['p', ','] },
  { id: 'nextFile', label: 'Next file', keys: ['f', ']'] },
  { id: 'prevFile', label: 'Previous file', keys: ['shift+f', '['] },
  { id: 'collapseFile', label: 'Collapse / expand current file', keys: ['x', 'shift+x'] },
  { id: 'nextPending', label: 'Next step with pending lines', keys: ['space', 'shift+n'] },
  { id: 'accept', label: 'Accept selection / focused hunk', keys: ['a', 'y'] },
  { id: 'defer', label: 'Put off for later', keys: ['d', 'shift+l'] },
  { id: 'reset', label: 'Reset to pending', keys: ['u', 'backspace'] },
  { id: 'acceptRest', label: 'Accept everything left, go to next step', keys: ['shift+a', 'mod+enter'] },
  { id: 'comment', label: 'New conversation about the selection', keys: ['c', '/'] },
  { id: 'rename', label: 'Rename the symbol under the pointer', keys: ['f2', 'shift+f6'] },
  { id: 'edit', label: 'Edit the focused hunk in place', keys: ['e', 'o'] },
  { id: 'toggleLayout', label: 'Toggle split / unified', keys: ['s', 'shift+s'] },
  { id: 'laterView', label: 'Go to issues left for later', keys: ['l', 'shift+d'] },
  { id: 'undo', label: 'Undo last mark', keys: ['z', 'mod+z'] },
  { id: 'reload', label: 'Reload diff from disk', keys: ['r', 'f5'] },
  { id: 'back', label: 'Back', keys: ['mod+-', 'mod+['] },
  { id: 'forward', label: 'Forward', keys: ['mod+=', 'mod+]'] },
  { id: 'chat', label: 'Talk about the whole change', keys: ['t', 'g'] },
  { id: 'find', label: 'Find in what is shown', keys: ['mod+f', 'alt+f'] },
  { id: 'findNext', label: 'Next match', keys: ['f3', 'mod+g'] },
  { id: 'findPrev', label: 'Previous match', keys: ['shift+f3', 'mod+shift+g'] },
  { id: 'searchProject', label: 'Search the whole project', keys: ['mod+shift+f', 'alt+shift+f'] },
  { id: 'settings', label: 'Settings', keys: ['mod+,', 'shift+,'] },
  { id: 'help', label: 'Keyboard help', keys: ['?', 'f1'] },
] as const;

export type ActionId = (typeof ACTIONS)[number]['id'];
export type Keybindings = Record<ActionId, string[]>;

export const defaultKeybindings = (): Keybindings => Object.fromEntries(ACTIONS.map((a) => [a.id, [...a.keys]])) as Keybindings;

const isMac = typeof navigator !== 'undefined' && /Mac/.test(navigator.platform);

/** Normalised combo of a key event, e.g. `mod+shift+a`, `space`, `?`. Null for bare modifiers. */
export function comboOf(e: KeyboardEvent): string | null {
  if (['Meta', 'Control', 'Alt', 'Shift'].includes(e.key)) return null;
  let key = e.key === ' ' ? 'space' : e.key.toLowerCase();
  if (e.altKey && e.code.startsWith('Key')) key = e.code.slice(3).toLowerCase();
  if (e.altKey && e.code.startsWith('Digit')) key = e.code.slice(5);
  const named = key.length > 1;
  const letter = /^[a-z]$/.test(key);
  const parts: string[] = [];
  if (isMac ? e.metaKey : e.ctrlKey) parts.push('mod');
  if (isMac && e.ctrlKey) parts.push('ctrl');
  if (e.altKey) parts.push('alt');
  if (e.shiftKey && (letter || named || parts.length)) parts.push('shift');
  if (e.shiftKey && key === '+' && parts.includes('mod')) key = '=';
  parts.push(key);
  return parts.join('+');
}

export function prettyCombo(c: string): string {
  const map: Record<string, string> = { mod: isMac ? '⌘' : 'Ctrl', ctrl: '⌃', alt: isMac ? '⌥' : 'Alt', shift: '⇧', arrowup: '↑', arrowdown: '↓', arrowleft: '←', arrowright: '→', enter: '↵', backspace: '⌫', space: 'Space' };
  return c.split('+').map((p) => map[p] ?? (p.length === 1 ? p.toUpperCase() : p.toUpperCase())).join(isMac ? '' : '+');
}
