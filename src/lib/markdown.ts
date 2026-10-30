import { Marked } from 'marked';

const escape = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

// Raw HTML in AI output is shown as text, never rendered.
const md = new Marked({ gfm: true, breaks: true, renderer: { html: ({ text }) => escape(text) } });

export function renderMarkdown(src: string): string {
  return md.parse(src, { async: false }) as string;
}
