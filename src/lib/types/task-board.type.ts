/** Where the dragged board card would land: its column (a status) and index in it. */
export type BoardSpot = { id: string; col: string; idx: number };

/** A card's place on the board: its column and its row in that column. */
export type CardSpot = { col: number; idx: number };
