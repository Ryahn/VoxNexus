/* ============================================================
   VOX domain models
   Shaped to map cleanly onto a future REST/WS API. UI never
   invents these shapes inline — mock data in /data conforms here.
   ============================================================ */

export type Presence = 'online' | 'idle' | 'dnd' | 'offline';

export type ChannelType =
  | 'text'
  | 'announcement'
  | 'voice'
  | 'stream'
  | 'forum'
  | 'calendar'
  | 'events'
  | 'tasks'
  | 'docs'
  | 'media'
  | 'poll'
  | 'applications'
  | 'recruitment';

export interface Role {
  id: string;
  name: string;
  /** rgb channels "r g b" so UI can apply opacity */
  color: string;
  /** display priority; higher sorts first in member list */
  rank: number;
  hoist: boolean;
}

export interface User {
  id: string;
  displayName: string;
  username: string;
  /** avatar seed → deterministic generated avatar */
  avatarSeed: string;
  accent: string; // "r g b"
  presence: Presence;
  status?: string;
  activity?: { kind: 'playing' | 'coding' | 'listening' | 'streaming'; label: string };
  bio?: string;
  bannerSeed?: string;
  memberSince?: string;
  roleIds: string[];
  pronouns?: string;
  mutuals?: number;
}

export interface Community {
  id: string;
  name: string;
  tag: string; // short 1-3 char rail glyph
  accent: string; // "r g b"
  unread?: number;
  mentions?: number;
  active?: boolean;
  bannerSeed?: string;
}

export interface Group {
  id: string;
  name: string;
  /** section header this group sits under in the group selector */
  section: string;
  icon?: string;
  unread?: boolean;
  restricted?: boolean; // permission-gated (staff only, etc.)
}

export interface Channel {
  id: string;
  groupId: string;
  categoryId: string;
  type: ChannelType;
  name: string;
  topic?: string;
  unread?: boolean;
  mentions?: number;
  muted?: boolean;
  locked?: boolean;
  /** for voice: users currently connected */
  connected?: string[];
  liveCount?: number;
}

export interface Category {
  id: string;
  groupId: string;
  name: string;
}

export interface Reaction {
  emoji: string;
  count: number;
  me?: boolean;
}

export interface Attachment {
  id: string;
  kind: 'image' | 'file';
  name: string;
  meta?: string; // "1440×900 · 284 KB"
  url?: string;
  thumbnailUrl?: string;
  hueA?: string;
  hueB?: string;
}

export interface Embed {
  kind: 'link';
  site: string;
  title: string;
  description: string;
  accent?: string;
}

export interface ThreadMeta {
  id: string;
  title: string;
  replyCount: number;
  participantIds: string[];
  lastReplyAt: string;
  following?: boolean;
}

export interface Message {
  id: string;
  channelId: string;
  authorId: string;
  ts: string; // display time e.g. "14:32"
  fullTs?: string; // hover tooltip
  content?: string;
  edited?: boolean;
  replyTo?: {
    messageId: string;
    authorId: string;
    excerpt: string;
    authorDisplayName?: string;
    deleted?: boolean;
  };
  reactions?: Reaction[];
  attachments?: Attachment[];
  embeds?: Embed[];
  code?: { lang: string; body: string };
  thread?: ThreadMeta;
  mentionsMe?: boolean;
  pinned?: boolean;
  system?: string; // system line text (joins, boosts)
}

export interface ThreadReply {
  id: string;
  authorId: string;
  ts: string;
  content: string;
  reactions?: Reaction[];
}

export type NotificationKind =
  | 'mention'
  | 'reply'
  | 'reaction'
  | 'announcement'
  | 'friend'
  | 'system';

export interface AppNotification {
  id: string;
  kind: NotificationKind;
  actorId?: string;
  title: string;
  body: string;
  ts: string;
  communityTag?: string;
  unread?: boolean;
}

export type SearchResultKind = 'message' | 'user' | 'channel' | 'community' | 'file' | 'command';

export interface SearchResult {
  id: string;
  kind: SearchResultKind;
  title: string;
  subtitle?: string;
  meta?: string;
  glyph?: string;
  accent?: string;
}
