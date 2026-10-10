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

/** An explorer row drag (util/explorer-drag.ts). */
export type RowDragOptions = {
  /** The pointerdown that may start a drag. */
  start: PointerEvent;
  /** The pointer has travelled far enough: the drag begins. */
  onstart: (e: PointerEvent) => void;
  /** The latest pointer position while dragging, at most once per frame. */
  onmove: (e: PointerEvent) => void;
  /** The pointer was released (`dropped`) or the drag was cancelled; also when no drag began. */
  onend: (dropped: boolean) => void;
};
