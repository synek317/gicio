/** The last repositories opened from the start screen (per machine, in local storage). */
export interface RecentRepos {
  repo: string;
  recent: string[];
}

const KEY = 'gicio.start';

export function loadRecent(): RecentRepos {
  try {
    const raw = JSON.parse(localStorage.getItem(KEY) ?? '{}');
    return { repo: raw.repo ?? '', recent: raw.recent ?? [] };
  } catch {
    return { repo: '', recent: [] };
  }
}

export function saveRecent(r: RecentRepos) {
  try {
    localStorage.setItem(KEY, JSON.stringify(r));
  } catch {
    /* private mode */
  }
}
