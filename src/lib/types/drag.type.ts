export type Point = { x: number; y: number };

export type PointerDrag = {
  move: (e: PointerEvent) => void;
  /** The pointer was released or cancelled. Runs after `cleanup`. */
  end: (e: PointerEvent) => void;
  /** Runs once when tracking stops, whether the pointer ended it or the owner did. */
  cleanup: () => void;
};

/** The floating copy of a card that follows the pointer. */
export type DragGhost = { move: (e: PointerEvent) => void; remove: () => void };
