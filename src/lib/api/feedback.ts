import { call } from './core.ts';

export type FeedbackKind = 'bug' | 'idea' | 'other';

export const feedbackApi = {
  send: (kind: FeedbackKind, body: string) => call<void>('feedback_send', { kind, body }),
};
