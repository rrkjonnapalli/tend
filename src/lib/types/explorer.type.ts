export type Leaf = { id: string; folder: string; name: string };

export type ExplorerItem = { kind: 'folder'; path: string } | { kind: 'leaf'; id: string };

export type ExplorerActions = {
  open: (item: ExplorerItem) => void;
  /** Resolves to the created path (a taken name gets a suffix). */
  createFolder: (input: { parent: string; name: string }) => Promise<string>;
  /** Notes: create a leaf inline. Left out for explorers that hold folders only. */
  createLeaf?: (input: { folder: string; name: string }) => Promise<void>;
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

export type NewKind = 'folder' | 'leaf';

/** One visible line of the tree. */
export type ExplorerRow = {
  key: string;
  item: ExplorerItem;
  name: string;
  depth: number;
  folder: string;
  isFolder: boolean;
  isNew?: NewKind;
};

/** The folders and leaves an explorer shows. */
export type ExplorerTree = { folders: string[]; leaves: Leaf[] };

/** An existing row being renamed, or a new row being named. */
export type ExplorerEdit = { key: string; folder: string; isNew?: NewKind; item?: ExplorerItem };

/** How a folder row looks while something is dragged over it. */
export type DropState = 'allowed' | 'blocked' | null;
