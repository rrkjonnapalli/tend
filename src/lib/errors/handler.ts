// The one place that shows errors: any promise rejection or thrown error nobody handled ends
// up here and is shown with its registry message. Components just `await`.
import { toAppError } from './app-error.ts';

type Show = (message: string) => void;

/** Starts listening; returns the function that stops it. */
export function installErrorHandler(show: Show): () => void {
  const report = (reason: unknown) => {
    // Keep the stack in the console; the user gets the plain message.
    console.error(reason);
    show(toAppError(reason).message);
  };
  const onRejection = (e: PromiseRejectionEvent) => {
    e.preventDefault();
    report(e.reason);
  };
  const onError = (e: ErrorEvent) => {
    report(e.error ?? e.message);
  };
  window.addEventListener('unhandledrejection', onRejection);
  window.addEventListener('error', onError);
  return () => {
    window.removeEventListener('unhandledrejection', onRejection);
    window.removeEventListener('error', onError);
  };
}
