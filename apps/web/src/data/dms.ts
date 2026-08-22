import type { Message } from '../types';

export interface DMConversation {
  userId: string;
  /** preview line shown in the DM list */
  preview: string;
  time: string;
  unread?: number;
  messages: Message[];
}

function m(
  id: string,
  uid: string,
  authorId: string,
  ts: string,
  content: string,
  extra: Partial<Message> = {},
): Message {
  return { id, channelId: `dm-${uid}`, authorId, ts, fullTs: `Today at ${ts}`, content, ...extra };
}

/* Direct-message threads. Keyed by the other participant's user id. */
export const dmConversations: Record<string, DMConversation> = {
  nova: {
    userId: 'nova',
    preview: 'Ship it. I’ll write the announcement.',
    time: '2m',
    unread: 2,
    messages: [
      m(
        'dn1',
        'nova',
        'nova',
        '10:31',
        'Hey — is the composer locale fix merged? Aya said it was blocking rc.1.',
      ),
      m(
        'dn2',
        'nova',
        'me',
        '10:33',
        'Just landed. `composer.json` is wired up and Aya has the strings.',
      ),
      m(
        'dn3',
        'nova',
        'nova',
        '10:34',
        'Perfect. And the scroll-jump thread — did the grid-columns fix stick?',
        {
          reactions: [{ emoji: '👀', count: 1, me: true }],
        },
      ),
      m(
        'dn4',
        'nova',
        'me',
        '10:35',
        'Yep, Dex re-ran the repro and closed it. rc.1 is clean on my end.',
      ),
      m('dn5', 'nova', 'nova', '10:36', 'Ship it. I’ll write the announcement.', {
        reactions: [{ emoji: '🚀', count: 1, me: true }],
      }),
    ],
  },
  kaito: {
    userId: 'kaito',
    preview: 'pushed the viewport subscribe, take a look',
    time: '18m',
    messages: [
      m(
        'dk1',
        'kaito',
        'kaito',
        '10:02',
        'pushed the viewport subscribe, take a look when you get a sec',
      ),
      m('dk2', 'kaito', 'kaito', '10:02', 'gateway CPU is way down on the stress account'),
      m('dk3', 'kaito', 'me', '10:12', 'huge. approving now — this unblocks the release'),
    ],
  },
  mira: {
    userId: 'mira',
    preview: 'contrast passes AA on every surface now 🎉',
    time: '1h',
    messages: [
      m('dm1', 'mira', 'mira', '09:20', 'token pass is up. role colors are all CSS vars now'),
      m('dm2', 'mira', 'mira', '09:21', 'contrast passes AA on every surface now 🎉'),
      m(
        'dm3',
        'mira',
        'me',
        '09:44',
        'beautiful. that means themes won’t break accessibility either',
      ),
    ],
  },
  aya: {
    userId: 'aya',
    preview: 'got the composer strings, 6 languages done',
    time: '3h',
    messages: [
      m('da1', 'aya', 'aya', '08:50', 'got the composer strings, 6 languages done so far'),
      m('da2', 'aya', 'me', '08:55', 'you’re a machine, thank you 🙏'),
    ],
  },
  ren: {
    userId: 'ren',
    preview: 'deep work, ping me after standup',
    time: '5h',
    messages: [
      m(
        'dr1',
        'ren',
        'ren',
        '07:30',
        'deep work today, ping me after standup if you need the compiler pass reviewed',
      ),
    ],
  },
};

export const dmList = Object.values(dmConversations);
