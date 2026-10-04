export type ReorderOptions = {
  keys: string[];
  key: string;
  min?: number;
  max?: number;
  preview: (keys: string[] | null) => void;
  drop: (keys: string[], index: number) => void;
};
