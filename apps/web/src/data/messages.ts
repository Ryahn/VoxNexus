import type { Message } from '../types';

/* Conversation for #development. Ordered oldest → newest.
   Consecutive messages by the same author within a short window
   are grouped by the renderer (author check), not encoded here. */
export const messages: Message[] = [
  {
    id: 'm0',
    channelId: 'ch-development',
    authorId: 'nova',
    ts: '09:02',
    fullTs: 'Today at 09:02',
    system: undefined,
    content:
      'Morning all. Cut `v0.7.0-rc.1` last night — presence rewrite is in. Please hammer on it today before we tag the release.',
    pinned: true,
    reactions: [
      { emoji: '🚀', count: 8, me: true },
      { emoji: '👀', count: 3 },
    ],
  },
  {
    id: 'm1',
    channelId: 'ch-development',
    authorId: 'kaito',
    ts: '09:05',
    fullTs: 'Today at 09:05',
    content:
      'The new heartbeat interval is way smoother. Idle → online transitions are basically instant now. Reconnect storms handled by the backoff jitter.',
    reactions: [{ emoji: '🔥', count: 5 }],
  },
  {
    id: 'm2',
    channelId: 'ch-development',
    authorId: 'kaito',
    ts: '09:05',
    content:
      'One thing though — presence for users in 100+ communities still fans out server-side. Might revisit.',
  },
  {
    id: 'm3',
    channelId: 'ch-development',
    authorId: 'ren',
    ts: '09:11',
    fullTs: 'Today at 09:11',
    replyTo: {
      messageId: 'm2',
      authorId: 'kaito',
      excerpt: 'presence for users in 100+ communities still fans out…',
    },
    content:
      'That’s the subscription model. I can move it to a lazy per-viewport subscribe. Drafting the query now:',
    code: {
      lang: 'typescript',
      body: `// only subscribe to presence for members actually on screen
function usePresenceViewport(memberIds: string[]) {
  const visible = useIntersecting(memberIds)
  return useSubscription(
    presenceChannel,
    { members: visible },
    { debounceMs: 120 },
  )
}`,
    },
    reactions: [
      { emoji: '🧠', count: 4, me: true },
      { emoji: '✅', count: 2 },
    ],
  },
  {
    id: 'm4',
    channelId: 'ch-development',
    authorId: 'mira',
    ts: '09:18',
    fullTs: 'Today at 09:18',
    content:
      'Pushed the token pass for the member list. Role colors now derive from CSS vars so theming won’t need a rebuild. Contrast checked against WCAG AA on the dark base.',
    edited: true,
  },
  {
    id: 'm5',
    channelId: 'ch-development',
    authorId: 'mira',
    ts: '09:19',
    content: 'Here’s the before/after on the presence dots — bigger hit area, still compact:',
    attachments: [
      {
        id: 'att-1',
        kind: 'image',
        name: 'presence-dots-compare.png',
        meta: '1440×720 · 218 KB',
        hueA: '54 210 205',
        hueB: '138 124 246',
      },
    ],
    reactions: [
      { emoji: '😍', count: 6 },
      { emoji: '♿', count: 2 },
    ],
  },
  {
    id: 'm6',
    channelId: 'ch-development',
    authorId: 'aya',
    ts: '09:24',
    fullTs: 'Today at 09:24',
    content:
      'Heads up @ryanc — the composer placeholder string isn’t in the locale bundle yet, so it falls back to English for 13 languages. Small fix but blocks the RC.',
    mentionsMe: true,
    reactions: [{ emoji: '🌍', count: 3 }],
  },
  {
    id: 'm7',
    channelId: 'ch-development',
    authorId: 'me',
    ts: '09:27',
    fullTs: 'Today at 09:27',
    replyTo: {
      messageId: 'm6',
      authorId: 'aya',
      excerpt: 'the composer placeholder string isn’t in the locale bundle…',
    },
    content:
      'On it — extracting all composer strings into `composer.json` now. Will ping you to translate once it’s up.',
    reactions: [{ emoji: '🙏', count: 2, me: false }],
  },
  {
    id: 'm8',
    channelId: 'ch-development',
    authorId: 'dex',
    ts: '09:33',
    fullTs: 'Today at 09:33',
    content:
      'Filed a repro for the scroll-jump when a thread opens. It’s the panel width transition fighting the message list. Thread below 👇',
    thread: {
      id: 't-scrolljump',
      title: 'Scroll jump when thread panel opens',
      replyCount: 4,
      participantIds: ['dex', 'kaito', 'me', 'mira'],
      lastReplyAt: '10:02',
      following: true,
    },
    reactions: [{ emoji: '🐛', count: 4 }],
  },
  {
    id: 'm9',
    channelId: 'ch-development',
    authorId: 'nova',
    ts: '09:41',
    fullTs: 'Today at 09:41',
    embeds: [
      {
        kind: 'link',
        site: 'nexus.dev',
        title: 'VOX v0.7 — Presence, Threads & Theming',
        description:
          'Release candidate notes: rewritten presence pipeline, right-side thread panel, CSS-variable theming, and 6 accessibility fixes.',
        accent: '54 210 205',
      },
    ],
    content: 'Draft release notes if anyone wants to proofread before we publish:',
  },
  {
    id: 'm10',
    channelId: 'ch-development',
    authorId: 'ren',
    ts: '09:52',
    fullTs: 'Today at 09:52',
    content:
      'Viewport presence subscribe is up. CPU on the gateway dropped ~40% for the stress account with 240 communities. 🎉',
    reactions: [
      { emoji: '📉', count: 7, me: true },
      { emoji: '🎉', count: 5 },
      { emoji: '🧊', count: 2 },
    ],
  },
  {
    id: 'm11',
    channelId: 'ch-development',
    authorId: 'kaito',
    ts: '09:53',
    replyTo: { messageId: 'm10', authorId: 'ren', excerpt: 'CPU on the gateway dropped ~40%…' },
    content: 'Merged. That’s the last blocker on my list for rc.1.',
  },
  {
    id: 'm12',
    channelId: 'ch-development',
    authorId: 'me',
    ts: '10:04',
    fullTs: 'Today at 10:04',
    content:
      'Composer strings extracted + wired to the loader. `composer.json` is in `locales/en/`. @aya it’s all yours whenever. Also added the send-shortcut hint that respects the user’s setting.',
    mentionsMe: false,
    reactions: [
      { emoji: '🌍', count: 2 },
      { emoji: '💜', count: 3 },
    ],
    edited: true,
  },
];

export function messagesForChannel(channelId: string): Message[] {
  return messages.filter((m) => m.channelId === channelId);
}
