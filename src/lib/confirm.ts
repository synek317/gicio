import { inTauri } from './api';

/** A yes/no question. In the app it is a native dialog: the webview's `window.confirm` is not reliable there. */
export async function confirm(message: string, title = 'gicio'): Promise<boolean> {
  if (!inTauri) return window.confirm(message);
  const { ask } = await import('@tauri-apps/plugin-dialog');
  return ask(message, { title, kind: 'warning' });
}
