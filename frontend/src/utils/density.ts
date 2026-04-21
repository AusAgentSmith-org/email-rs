export const DENSITY_MIN = 0;
export const DENSITY_MAX = 8;
export const DENSITY_DEFAULT = 4;

export const DENSITY_LABELS = [
  'Compact', 'Dense', 'Snug', 'Tight', 'Normal',
  'Relaxed', 'Airy', 'Cozy', 'Spacious',
];

const DENSITY_VARS = [
  { rowHeight: 28, paddingV:  3, paddingH:  8, sidebarItemH: 22 },
  { rowHeight: 34, paddingV:  4, paddingH: 10, sidebarItemH: 24 },
  { rowHeight: 40, paddingV:  6, paddingH: 12, sidebarItemH: 26 },
  { rowHeight: 46, paddingV:  8, paddingH: 14, sidebarItemH: 29 },
  { rowHeight: 52, paddingV: 10, paddingH: 16, sidebarItemH: 32 },
  { rowHeight: 57, paddingV: 11, paddingH: 17, sidebarItemH: 34 },
  { rowHeight: 62, paddingV: 12, paddingH: 18, sidebarItemH: 36 },
  { rowHeight: 67, paddingV: 13, paddingH: 19, sidebarItemH: 37 },
  { rowHeight: 72, paddingV: 14, paddingH: 20, sidebarItemH: 38 },
];

export function applyDensity(level: number): void {
  const v = DENSITY_VARS[Math.max(DENSITY_MIN, Math.min(DENSITY_MAX, level))];
  const root = document.documentElement;
  root.style.setProperty('--row-height',      `${v.rowHeight}px`);
  root.style.setProperty('--row-padding-v',   `${v.paddingV}px`);
  root.style.setProperty('--row-padding-h',   `${v.paddingH}px`);
  root.style.setProperty('--sidebar-item-h',  `${v.sidebarItemH}px`);
  root.setAttribute('data-density-level', String(level));
}
