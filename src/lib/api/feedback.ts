import type { FeedbackKind } from '#lib/types/feedback.type.ts';
import { call } from './core.ts';

export const feedbackAPI = {
  send: (input: { kind: FeedbackKind; body: string }) => call<void>('feedback_send', { args: input }),
};
