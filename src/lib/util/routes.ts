// URLs for every page, so links are built in one place.

const q = (path: string) => (path ? `?f=${encodeURIComponent(path)}` : '');

export const routes = {
  notesFolder: (path: string) => `/notes${q(path)}`,
  note: (id: string, edit = false) => `/notes/${id}${edit ? '?edit=1' : ''}`,
  tasksFolder: (path: string) => `/tasks${q(path)}`,
  task: (id: string) => `/tasks/${id}`,
  person: (id: string) => `/people/${id}`,
};
