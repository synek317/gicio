import { Marked, Renderer } from 'marked';
import { linkCodeSpan, linkPathsInHtml } from './path-links';

const escape = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

/** Text inside a markdown link keeps that link; file mentions there are not linked again. */
let inLink = 0;

// Raw HTML in AI output is shown as text, never rendered. File mentions become clickable.
const md = new Marked({
  gfm: true,
  breaks: true,
  renderer: {
    html: ({ text }) => escape(text),
    link(token) {
      inLink++;
      try {
        return Renderer.prototype.link.call(this, token);
      } finally {
        inLink--;
      }
    },
    text(token) {
      if (('tokens' in token && token.tokens) || inLink) return false;
      return linkPathsInHtml(Renderer.prototype.text.call(this, token));
    },
    codespan(token) {
      const html = Renderer.prototype.codespan.call(this, token);
      return inLink ? html : linkCodeSpan(token.text, html);
    },
  },
});

export function renderMarkdown(src: string): string {
  return md.parse(src, { async: false }) as string;
}
