import { compileFind, matchRanges, type FindSpec } from './find-query';
import type { FindResult, FindTarget } from './find.svelte';

/** Text the find bar searches in the diff: code lines (not the buttons inside them), reference
 *  previews and the lines of an in-place editor. */
const ROOTS = '.line code.code, .ref-row code, .inline-editor .cm-line';
const MAX_MATCHES = 5000;
const supportsHighlights = typeof CSS !== 'undefined' && 'highlights' in CSS;

let spec: FindSpec | null = null;
let report: (r: FindResult) => void = () => {};
let matches: Range[] = [];
let current = -1;
let observer: MutationObserver | null = null;
let timer: ReturnType<typeof setTimeout> | undefined;

const scroller = () => document.getElementById('diff-scroll');

function rangesIn(el: Element, re: RegExp, wholeWord: boolean): Range[] {
  const nodes: Text[] = [];
  const starts: number[] = [];
  let text = '';
  const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT, {
    acceptNode: (n) => (n.parentElement?.closest('button') ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_ACCEPT),
  });
  for (let n = walker.nextNode(); n; n = walker.nextNode()) {
    starts.push(text.length);
    nodes.push(n as Text);
    text += (n as Text).data;
  }
  /** Text node and offset of a position; an end position stays in the node it ends in. */
  const at = (pos: number, end: boolean): [Text, number] => {
    let i = nodes.length - 1;
    while (i > 0 && (end ? starts[i] >= pos : starts[i] > pos)) i--;
    return [nodes[i], pos - starts[i]];
  };
  return matchRanges(text, re, wholeWord).map(([from, to]) => {
    const r = document.createRange();
    r.setStart(...at(from, false));
    r.setEnd(...at(to, true));
    return r;
  });
}

function collect(s: FindSpec): Range[] {
  const root = scroller();
  if (!root) return [];
  const re = compileFind(s);
  const out: Range[] = [];
  for (const el of root.querySelectorAll(ROOTS)) {
    out.push(...rangesIn(el, re, s.wholeWord));
    if (out.length >= MAX_MATCHES) break;
  }
  return out.slice(0, MAX_MATCHES);
}

/** Top and bottom of the part of the scroller not covered by its sticky header. */
function visibleBand(sc: HTMLElement): [number, number] {
  const box = sc.getBoundingClientRect();
  const head = sc.querySelector('.chunk-head')?.getBoundingClientRect().bottom ?? box.top;
  return [Math.max(box.top, head), box.bottom];
}

/** The first match not above the visible part. */
function firstOnScreen(): number {
  const sc = scroller();
  if (!sc || !matches.length) return matches.length ? 0 : -1;
  const [top] = visibleBand(sc);
  const i = matches.findIndex((r) => r.getBoundingClientRect().bottom > top);
  return i < 0 ? 0 : i;
}

function reveal() {
  const sc = scroller();
  const r = matches[current];
  if (!sc || !r) return;
  const rect = r.getBoundingClientRect();
  const [top, bottom] = visibleBand(sc);
  if (rect.top < top || rect.bottom > bottom) sc.scrollBy({ top: rect.top - (top + bottom) / 2 });
  if (!supportsHighlights) {
    const sel = window.getSelection();
    sel?.removeAllRanges();
    sel?.addRange(r);
  }
}

function paint() {
  if (!supportsHighlights) return;
  CSS.highlights.set('find-match', new Highlight(...matches));
  CSS.highlights.set('find-current', new Highlight(...(matches[current] ? [matches[current]] : [])));
}

function update(keepCurrent: boolean) {
  matches = spec ? collect(spec) : [];
  current = !matches.length ? -1 : keepCurrent && current >= 0 && current < matches.length ? current : firstOnScreen();
  paint();
  report({ count: matches.length, current });
}

/** The diff re-renders (marks, another step, expanded lines): look again, without scrolling. */
function watch() {
  const sc = scroller();
  observer?.disconnect();
  if (!sc) return;
  observer = new MutationObserver(() => {
    clearTimeout(timer);
    timer = setTimeout(() => update(true), 120);
  });
  observer.observe(sc, { childList: true, subtree: true, characterData: true });
}

export const domFindTarget: FindTarget = {
  show(s, r) {
    spec = s;
    report = r;
    clearTimeout(timer);
    if (!s) {
      observer?.disconnect();
      observer = null;
      matches = [];
      current = -1;
      paint();
      return;
    }
    watch();
    update(false);
    reveal();
  },
  step(dir) {
    if (!matches.length) return;
    current = (current + dir + matches.length) % matches.length;
    paint();
    reveal();
    report({ count: matches.length, current });
  },
  selectedText: () => window.getSelection()?.toString().trim() ?? '',
};
