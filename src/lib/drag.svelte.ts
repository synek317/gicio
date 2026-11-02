/** Gutter drag-selection state shared between gutters; released on any mouseup. */
export const drag = $state({ active: false });

if (typeof window !== 'undefined') window.addEventListener('mouseup', () => (drag.active = false));
