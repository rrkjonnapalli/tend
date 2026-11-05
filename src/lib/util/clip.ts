// The paste rule of the phone's cut and paste (tend-mobile-spec.md §8).

/** Items can be pasted in `target` unless every one of them is already there. */
export const canPaste = ({ from, target }: { from: string[]; target: string }) => from.some((folder) => folder !== target);
