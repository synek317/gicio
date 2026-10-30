export function scrollToLine(path: string, side: 'old' | 'new', line: number) {
  const el = document.querySelector(`[data-line="${CSS.escape(`${path}|${side}|${line}`)}"]`);
  el?.scrollIntoView({ block: 'center', behavior: 'smooth' });
  el?.classList.add('flash');
  setTimeout(() => el?.classList.remove('flash'), 1200);
}
