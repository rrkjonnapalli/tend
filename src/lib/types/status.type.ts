export type Status = { value: string; label: string; color: string; position: number; isDefault: boolean };

/** Where a status sits in the list: the first and last ones can't be moved. */
export type StatusPosition = 'first' | 'middle' | 'last';
