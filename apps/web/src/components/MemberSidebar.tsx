import { type CommunityMemberResponse, listCommunityMembers } from '@voxnexus/api-client';
import { useEffect, useMemo, useState } from 'react';
import { buildMemberSections } from '../data/members';
import { useUI } from '../store';
import type { User } from '../types';
import { MemberItem } from './MemberItem';

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

function memberToUser(m: CommunityMemberResponse): User {
  const name = m.nickname.trim() || m.display_name || 'Member';
  return {
    id: m.account_id,
    displayName: name,
    username: m.account_id.slice(0, 8),
    avatarSeed: m.account_id,
    accent: '54 210 205',
    presence: 'online',
    roleIds: m.role === 'owner' ? ['owner'] : [],
    memberSince: m.joined_at,
  };
}

export function MemberSidebar() {
  const activeCommunity = useUI((s) => s.activeCommunity);
  const mockSections = useMemo(() => buildMemberSections(), []);
  const [liveMembers, setLiveMembers] = useState<User[] | null>(null);

  useEffect(() => {
    if (!UUID_RE.test(activeCommunity)) {
      setLiveMembers(null);
      return;
    }
    let cancelled = false;
    void (async () => {
      const result = await listCommunityMembers({
        path: { community_id: activeCommunity },
        query: { limit: 100 },
      });
      if (cancelled) return;
      if (result.data?.items) {
        setLiveMembers(result.data.items.map(memberToUser));
      } else {
        setLiveMembers([]);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [activeCommunity]);

  const sections =
    liveMembers === null
      ? mockSections
      : [
          {
            key: 'members',
            label: 'MEMBERS',
            color: undefined as string | undefined,
            members: liveMembers,
          },
        ];

  return (
    <aside className="flex h-full w-60 shrink-0 flex-col border-l border-line/70 bg-panel">
      <div className="flex h-12 shrink-0 items-center border-b border-line/70 px-3">
        <span className="font-sans text-[11px] font-semibold uppercase tracking-[0.14em] text-ink-3">
          Members
        </span>
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto px-2 py-2">
        {sections.map((sec) => (
          <div key={sec.key} className="mb-3">
            <div className="flex items-center gap-1.5 px-2 pb-1">
              <span
                className="kicker"
                style={sec.color ? { color: `rgb(${sec.color})` } : undefined}
              >
                {sec.label}
              </span>
              <span className="font-mono text-3xs text-ink-4">— {sec.members.length}</span>
            </div>
            <div className="flex flex-col gap-px">
              {sec.members.map((u) => (
                <MemberItem key={u.id} user={u} />
              ))}
            </div>
          </div>
        ))}
      </div>
    </aside>
  );
}
