import type { Role } from '../types';

export const roles: Record<string, Role> = {
  founder: { id: 'founder', name: 'Founder', color: '240 97 168', rank: 100, hoist: true },
  admin: { id: 'admin', name: 'Administrator', color: '54 210 205', rank: 90, hoist: true },
  maintainer: { id: 'maintainer', name: 'Maintainer', color: '138 124 246', rank: 80, hoist: true },
  mod: { id: 'mod', name: 'Moderator', color: '99 179 237', rank: 70, hoist: true },
  contributor: {
    id: 'contributor',
    name: 'Contributor',
    color: '99 202 130',
    rank: 50,
    hoist: false,
  },
  translator: { id: 'translator', name: 'Translator', color: '240 180 41', rank: 40, hoist: false },
  member: { id: 'member', name: 'Member', color: '141 152 173', rank: 10, hoist: false },
};

/** highest-ranked hoisted role → member list grouping + name color */
export function topRole(roleIds: string[]): Role {
  return roleIds
    .map((id) => roles[id])
    .filter(Boolean)
    .sort((a, b) => b.rank - a.rank)[0];
}

export function nameColor(roleIds: string[]): string {
  const r = roleIds
    .map((id) => roles[id])
    .filter(Boolean)
    .sort((a, b) => b.rank - a.rank)[0];
  return r ? r.color : '221 228 240';
}
