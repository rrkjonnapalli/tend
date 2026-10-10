/** Work to finish before the app quits (see util/before-quit.ts). */
export type QuitHook = () => Promise<void> | void;
