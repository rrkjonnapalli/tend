// Whether a `tend_enums` row may be used (`tend_enums.status`): a blocked tag can't be added to
// anything. Same values as src-tauri/src/enums.
export const ENUM_STATUS = {
  active: 'ACTIVE',
  blocked: 'BLOCKED',
} as const;

export type EnumStatus = (typeof ENUM_STATUS)[keyof typeof ENUM_STATUS];
