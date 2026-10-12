// What a reference is (`refs.kind`). Same values as src-tauri/src/enums `RefKind`.
export const REF_KIND = {
  link: 'LINK',
  text: 'TEXT',
} as const;

export type RefKind = (typeof REF_KIND)[keyof typeof REF_KIND];
