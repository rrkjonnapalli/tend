export type Leaf = { id: string; folder: string; name: string };

export type ExplorerItem = { kind: 'folder'; path: string } | { kind: 'leaf'; id: string };

export type ExplorerActions = {
  open: (item: ExplorerItem) => void;
  createFolder: (parent: string, name: string) => Promise<void>;
  /** Notes: create a leaf inline. Left out for explorers that hold folders only. */
  createLeaf?: (folder: string, name: string) => Promise<void>;
  /** The "new item" toolbar button when there is no inline leaf creation (tasks). */
  newItem?: (folder: string) => void;
  rename: (item: ExplorerItem, name: string) => Promise<void>;
  move: (item: ExplorerItem, target: string) => Promise<void>;
  remove: (item: ExplorerItem) => Promise<void>;
  reload: () => Promise<void>;
  /** "5 notes and 2 folders" for the delete dialog. */
  contents: (path: string) => string;
  /** When set, the root row can be renamed (its menu has Rename only). */
  renameRoot?: (name: string) => Promise<void>;
};
