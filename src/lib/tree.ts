import type { FileDiff } from './types';

export interface TreeNode { name: string; path: string; file: FileDiff | null; children: TreeNode[]; files: FileDiff[] }

/** Directory tree of changed files; single-child directory chains are collapsed (`src/a/b`). */
export function buildTree(files: FileDiff[]): TreeNode {
  const root: TreeNode = { name: '', path: '', file: null, children: [], files: [] };
  for (const f of files) {
    let node = root;
    const parts = f.path.split('/');
    parts.forEach((name, i) => {
      node.files.push(f);
      const path = parts.slice(0, i + 1).join('/');
      let child = node.children.find((c) => c.path === path);
      if (!child) {
        child = { name, path, file: i === parts.length - 1 ? f : null, children: [], files: [] };
        node.children.push(child);
      }
      node = child;
    });
    node.files.push(f);
  }
  const collapse = (n: TreeNode): TreeNode => {
    n.children = n.children.map(collapse);
    if (!n.file && n.children.length === 1 && !n.children[0].file && n.path) {
      const c = n.children[0];
      return { ...c, name: `${n.name}/${c.name}` };
    }
    return n;
  };
  const sort = (n: TreeNode) => {
    n.children.sort((a, b) => Number(!!a.file) - Number(!!b.file) || a.name.localeCompare(b.name));
    n.children.forEach(sort);
  };
  collapse(root);
  sort(root);
  return root;
}
