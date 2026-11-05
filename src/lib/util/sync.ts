// How a sync remote is shown on the phone (Settings, the Replace token sheet).

/** A remote's URL without its scheme and `.git`: 'github.com/you/notes'. */
export const repoLabel = (url: string) => url.trim().replace(/^https?:\/\//i, '').replace(/\.git$/i, '');
