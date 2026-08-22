import type { User } from '../types';
import { roles, topRole } from './roles';
import { userList } from './users';

export interface MemberSection {
  key: string;
  label: string;
  color?: string;
  members: User[];
}

/* Build role-hoisted sections like Guilded/Discord:
   hoisted roles first (Founder, Admin, …), then ONLINE, then OFFLINE.
   Online members with a hoisted role appear under their role only. */
export function buildMemberSections(): MemberSection[] {
  const online = userList.filter((u) => u.presence !== 'offline');
  const offline = userList.filter((u) => u.presence === 'offline');

  const hoisted = Object.values(roles)
    .filter((r) => r.hoist)
    .sort((a, b) => b.rank - a.rank);

  const claimed = new Set<string>();
  const sections: MemberSection[] = [];

  for (const role of hoisted) {
    const inRole = online.filter((u) => topRole(u.roleIds)?.id === role.id);
    if (!inRole.length) continue;
    inRole.forEach((u) => claimed.add(u.id));
    sections.push({
      key: role.id,
      label: role.name.toUpperCase(),
      color: role.color,
      members: sortByName(inRole),
    });
  }

  const restOnline = online.filter((u) => !claimed.has(u.id));
  if (restOnline.length) {
    sections.push({ key: 'online', label: 'ONLINE', members: sortByName(restOnline) });
  }
  if (offline.length) {
    sections.push({ key: 'offline', label: 'OFFLINE', members: sortByName(offline) });
  }
  return sections;
}

function sortByName(list: User[]): User[] {
  return [...list].sort((a, b) => a.displayName.localeCompare(b.displayName));
}

export const onlineCount = userList.filter((u) => u.presence !== 'offline').length;
export const totalCount = userList.length;
