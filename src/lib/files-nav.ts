import { app } from './state.svelte';

const scroller = () => document.getElementById('diff-scroll');
const headHeight = () => document.querySelector<HTMLElement>('.chunk-head')?.offsetHeight ?? 0;

/** Key of a file block in the current view, for its collapsed state. */
export function blockKey(path: string): string {
  const v = app.view;
  const id = v?.kind === 'chunk' ? v.id : v?.kind ?? '';
  return `${id}|${path}`;
}

/** The file block at the top of the viewport (index into `.file-block` elements). */
function currentIndex(blocks: HTMLElement[]): number {
  const top = (scroller()?.scrollTop ?? 0) + headHeight() + 8;
  let idx = 0;
  blocks.forEach((b, i) => {
    if (b.offsetTop <= top) idx = i;
  });
  return idx;
}

export function jumpFile(delta: number) {
  const sc = scroller();
  const blocks = [...document.querySelectorAll<HTMLElement>('.file-block')];
  if (!sc || !blocks.length) return;
  const target = blocks[currentIndex(blocks) + delta];
  if (!target) return app.toast(delta > 0 ? 'Last file of this step. Space goes to the next step.' : 'First file of this step.');
  sc.scrollTo({ top: target.offsetTop - headHeight(), behavior: 'smooth' });
  const path = target.dataset.path;
  const first = app.viewHunks.find((h) => h.path === path);
  if (first) app.focusedHunk = first.id;
}

export function toggleCurrentFile() {
  const blocks = [...document.querySelectorAll<HTMLElement>('.file-block')];
  const path = blocks[currentIndex(blocks)]?.dataset.path;
  if (path) setCollapsed(path, !app.collapsed[blockKey(path)]);
}

export function setCollapsed(path: string, on: boolean) {
  app.collapsed = { ...app.collapsed, [blockKey(path)]: on };
}
