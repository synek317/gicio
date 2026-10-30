/** Gutter drag-selection state shared between gutters; released on any mouseup. */
export const drag = { active: false };

if (typeof window !== 'undefined') window.addEventListener('mouseup', () => (drag.active = false));
