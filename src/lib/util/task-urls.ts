// Search params of the task list, and URLs built from them.
import type { Crumb } from '#lib/types/breadcrumb.type.ts';
import { routes } from './routes.ts';

export const TASK_PARAMS = { folder: 'f', view: 'v', from: 'from', to: 'to', add: 'add' } as const;

/** `{ f: folder }`, or nothing for the root folder. */
export function folderParam(folder: string): Record<string, string> {
  if (!folder) return {};
  return { [TASK_PARAMS.folder]: folder };
}

/** The task list with these search params. */
export function tasksURL(params: Record<string, string>): string {
  const query = new URLSearchParams(params).toString();
  if (!query) return routes.tasks;
  return `${routes.tasks}?${query}`;
}

/** The task list of `folder` with its add field focused. */
export const newTaskURL = (folder: string) => tasksURL({ ...folderParam(folder), [TASK_PARAMS.add]: '1' });

/** Root, then one crumb per folder down to `path`, each linking to its task list. */
export function folderCrumbs({ rootName, path }: { rootName: string; path: string }): Crumb[] {
  const crumbs: Crumb[] = [{ label: rootName, href: routes.tasksFolder('') }];
  const segs = path ? path.split('/') : [];
  segs.forEach((seg, i) => {
    const upTo = segs.slice(0, i + 1).join('/');
    crumbs.push({ label: seg, href: routes.tasksFolder(upTo) });
  });
  return crumbs;
}
