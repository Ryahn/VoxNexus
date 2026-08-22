import type { Community } from '../types';

export const communities: Community[] = [
  {
    id: 'nexus',
    name: 'Project Nexus',
    tag: 'NX',
    accent: '54 210 205',
    active: true,
    bannerSeed: 'teal-1',
  },
  {
    id: 'aurora',
    name: 'Aurora Collective',
    tag: 'AU',
    accent: '138 124 246',
    unread: 3,
    bannerSeed: 'violet-3',
  },
  { id: 'foss', name: 'FOSS Guild', tag: 'FG', accent: '99 202 130', mentions: 2, unread: 8 },
  { id: 'nightcity', name: 'Night City GG', tag: 'NC', accent: '240 97 168', unread: 12 },
  { id: 'synth', name: 'Synthwave Lounge', tag: 'SY', accent: '240 180 41' },
  { id: 'orbital', name: 'Orbital Ops', tag: 'OR', accent: '99 179 237', mentions: 1 },
  { id: 'design', name: 'Design Union', tag: 'DU', accent: '54 210 205' },
];
