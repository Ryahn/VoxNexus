import type { Category, Channel, Group } from '../types';

/* Groups (sub-communities) inside Project Nexus.
   Grouped under section headers in the GroupSelector. */
export const groups: Group[] = [
  { id: 'g-community', name: 'Community', section: 'COMMUNITY', icon: 'home' },
  { id: 'g-dev', name: 'Development', section: 'BUILD', icon: 'code', unread: true },
  { id: 'g-design', name: 'Design Lab', section: 'BUILD', icon: 'palette' },
  { id: 'g-translation', name: 'Translation Team', section: 'BUILD', icon: 'languages' },
  { id: 'g-staff', name: 'Staff', section: 'INTERNAL', icon: 'shield', restricted: true },
];

/* Categories belong to a group. */
export const categories: Category[] = [
  { id: 'c-dev-general', groupId: 'g-dev', name: 'GENERAL' },
  { id: 'c-dev-project', groupId: 'g-dev', name: 'PROJECT' },
  { id: 'c-dev-voice', groupId: 'g-dev', name: 'VOICE' },
  // other groups (for completeness / switching)
  { id: 'c-com-main', groupId: 'g-community', name: 'LOBBY' },
  { id: 'c-staff-main', groupId: 'g-staff', name: 'STAFF' },
];

/* Channels belong to a category (and thus a group). */
export const channels: Channel[] = [
  // Development ▸ GENERAL
  {
    id: 'ch-development',
    groupId: 'g-dev',
    categoryId: 'c-dev-general',
    type: 'text',
    name: 'development',
    topic: 'Core app work — frontend, realtime, infra. Keep it shippable.',
    unread: true,
  },
  {
    id: 'ch-screenshots',
    groupId: 'g-dev',
    categoryId: 'c-dev-general',
    type: 'media',
    name: 'screenshots',
    topic: 'Drop progress shots and mockups.',
    mentions: 2,
    unread: true,
  },
  {
    id: 'ch-ideas',
    groupId: 'g-dev',
    categoryId: 'c-dev-general',
    type: 'forum',
    name: 'ideas',
    topic: 'Proposals and RFCs.',
  },
  {
    id: 'ch-announcements',
    groupId: 'g-dev',
    categoryId: 'c-dev-general',
    type: 'announcement',
    name: 'announcements',
    topic: 'Release notes and important updates.',
    muted: true,
  },
  // Development ▸ PROJECT
  {
    id: 'ch-tasks',
    groupId: 'g-dev',
    categoryId: 'c-dev-project',
    type: 'tasks',
    name: 'Tasks',
    topic: 'Sprint board',
  },
  {
    id: 'ch-calendar',
    groupId: 'g-dev',
    categoryId: 'c-dev-project',
    type: 'calendar',
    name: 'Calendar',
  },
  {
    id: 'ch-docs',
    groupId: 'g-dev',
    categoryId: 'c-dev-project',
    type: 'docs',
    name: 'Documentation',
  },
  // Development ▸ VOICE
  {
    id: 'ch-dev-room',
    groupId: 'g-dev',
    categoryId: 'c-dev-voice',
    type: 'voice',
    name: 'Development Room',
    connected: ['kaito', 'ren', 'dex'],
    liveCount: 3,
  },
  {
    id: 'ch-afk',
    groupId: 'g-dev',
    categoryId: 'c-dev-voice',
    type: 'voice',
    name: 'AFK',
    connected: [],
  },
];

export const DEFAULT_CHANNEL = 'ch-development';
export const DEFAULT_GROUP = 'g-dev';
