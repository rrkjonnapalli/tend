export type ReorderOptions = {
  keys: string[];
  key: string;
  min?: number;
  max?: number;
  preview: (keys: string[] | null) => void;
  drop: (keys: string[], index: number) => void;
};

/** A finished drag: the ids in their new order and where the moved one landed. */
export type DroppedOrder = { order: string[]; index: number };
