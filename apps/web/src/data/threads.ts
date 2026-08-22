import type { Message, ThreadReply } from '../types';

/** The root message a thread was spun off from (mirrors m8). */
export const threadRoot: Message = {
  id: 'm8',
  channelId: 'ch-development',
  authorId: 'dex',
  ts: '09:33',
  fullTs: 'Today at 09:33',
  content:
    'Filed a repro for the scroll-jump when a thread opens. It’s the panel width transition fighting the message list.',
};

export const threadReplies: ThreadReply[] = [
  {
    id: 'tr1',
    authorId: 'kaito',
    ts: '09:38',
    content:
      'Confirmed. The list re-measures mid-transition. We should freeze scroll anchor until the panel settles.',
    reactions: [{ emoji: '✅', count: 2 }],
  },
  {
    id: 'tr2',
    authorId: 'me',
    ts: '09:44',
    content:
      'Easiest fix: animate `grid-template-columns` instead of width, and pin the scroll to the last read message id.',
  },
  {
    id: 'tr3',
    authorId: 'mira',
    ts: '09:51',
    content: 'Grid columns also fixes the sub-pixel shimmer on the divider. Let’s do that.',
    reactions: [{ emoji: '💯', count: 3, me: true }],
  },
  {
    id: 'tr4',
    authorId: 'dex',
    ts: '10:02',
    content: 'Nice — I’ll re-run the repro once it lands and close the issue if it’s clean.',
  },
];
