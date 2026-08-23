import { type CommunityResponse, listCommunities } from '@voxnexus/api-client';
import { Bell, Compass, Home, Plus } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useAuth } from '../auth';
import { useMeta } from '../meta';
import { useUI } from '../store';
import type { Community } from '../types';
import { CommunityIcon } from './CommunityIcon';
import { CreateCommunityModal } from './CreateCommunityModal';
import { JoinCommunityModal } from './JoinCommunityModal';
import { Tooltip } from './ui/Tooltip';

function toRailCommunity(c: CommunityResponse): Community {
  const tag = c.name
    .split(/\s+/)
    .map((part) => part[0] ?? '')
    .join('')
    .slice(0, 2)
    .toUpperCase();
  return {
    id: c.id,
    name: c.name,
    tag: tag || 'C',
    accent: '54 210 205',
  };
}

export function CommunityRail() {
  const active = useUI((s) => s.activeCommunity);
  const setCommunity = useUI((s) => s.setCommunity);
  const setNotifOpen = useUI((s) => s.setNotifOpen);
  const notifOpen = useUI((s) => s.notifOpen);
  const setCreateOpen = useUI((s) => s.setCreateCommunityOpen);
  const setJoinOpen = useUI((s) => s.setJoinCommunityOpen);
  const meta = useMeta();
  const { session } = useAuth();
  const mode = meta?.community_creation_mode;
  const hideCreateDiscover =
    mode === 'single' || (mode === 'admin_only' && !session.account.is_instance_admin);
  const [communities, setCommunities] = useState<Community[]>([]);

  const refresh = useCallback(async () => {
    const result = await listCommunities();
    if (result.data?.communities) {
      setCommunities(result.data.communities.map(toRailCommunity));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <nav
      aria-label="Communities"
      className="relative z-20 flex h-full w-[68px] flex-col items-center gap-2 border-r border-line/70 bg-rail py-3"
    >
      <div className="relative flex items-center justify-center">
        <span
          aria-hidden
          className={`absolute -left-3 w-1 rounded-r-full bg-accent transition-all duration-200 ${
            active === 'home' ? 'h-7 opacity-100' : 'h-0 opacity-0'
          }`}
        />
        <Tooltip label="Home & Direct Messages" side="right">
          <button
            type="button"
            onClick={() => setCommunity('home')}
            aria-label="Home and Direct Messages"
            className={`group grid h-11 w-11 place-items-center rounded-[34%] border transition-colors duration-150 ${
              active === 'home'
                ? 'border-accent/50 bg-accent/15 text-accent'
                : 'border-line-2/60 bg-surface/70 text-ink-2 hover:border-accent/40 hover:text-accent'
            }`}
          >
            <Home size={19} strokeWidth={1.9} />
          </button>
        </Tooltip>
      </div>

      <div className="my-1 h-px w-8 bg-line-2/70" />

      <div className="no-scrollbar flex flex-1 flex-col items-center gap-2.5 overflow-y-auto">
        {communities.map((c) => (
          <CommunityIcon
            key={c.id}
            community={c}
            active={active === c.id}
            onClick={() => setCommunity(c.id)}
          />
        ))}

        {!hideCreateDiscover ? (
          <div className="mt-1 flex flex-col items-center gap-2.5">
            <Tooltip label="Create a community" side="right">
              <button
                type="button"
                aria-label="Create a community"
                onClick={() => setCreateOpen(true)}
                className="grid h-11 w-11 place-items-center rounded-[50%] border border-dashed border-line-2/80 text-ink-2 transition-all duration-200 ease-swift hover:rounded-[34%] hover:border-accent/60 hover:bg-accent/10 hover:text-accent"
              >
                <Plus size={20} strokeWidth={2} />
              </button>
            </Tooltip>
            <Tooltip label="Join a community" side="right">
              <button
                type="button"
                aria-label="Join a community"
                onClick={() => setJoinOpen(true)}
                className="grid h-11 w-11 place-items-center rounded-[50%] border border-dashed border-line-2/80 text-ink-2 transition-all duration-200 ease-swift hover:rounded-[34%] hover:border-accent-2/60 hover:bg-accent-2/10 hover:text-accent-2"
              >
                <Compass size={19} strokeWidth={1.9} />
              </button>
            </Tooltip>
          </div>
        ) : mode !== 'single' ? (
          <div className="mt-1 flex flex-col items-center gap-2.5">
            <Tooltip label="Join a community" side="right">
              <button
                type="button"
                aria-label="Join a community"
                onClick={() => setJoinOpen(true)}
                className="grid h-11 w-11 place-items-center rounded-[50%] border border-dashed border-line-2/80 text-ink-2 transition-all duration-200 ease-swift hover:rounded-[34%] hover:border-accent-2/60 hover:bg-accent-2/10 hover:text-accent-2"
              >
                <Compass size={19} strokeWidth={1.9} />
              </button>
            </Tooltip>
          </div>
        ) : null}
      </div>

      <div className="mt-1 flex flex-col items-center gap-2">
        <div className="h-px w-8 bg-line-2/70" />
        <Tooltip label="Notifications" side="right" kbd="Ctrl B">
          <button
            type="button"
            aria-label="Notifications"
            onClick={() => setNotifOpen(!notifOpen)}
            className={`relative grid h-10 w-10 place-items-center rounded-[34%] transition-colors duration-150 ${
              notifOpen
                ? 'bg-surface-active text-accent'
                : 'text-ink-2 hover:bg-surface-hover hover:text-ink'
            }`}
          >
            <Bell size={18} strokeWidth={1.9} />
          </button>
        </Tooltip>
      </div>

      <CreateCommunityModal
        onCreated={(id) => {
          void refresh().then(() => setCommunity(id));
        }}
      />
      <JoinCommunityModal
        onJoined={(id) => {
          void refresh().then(() => setCommunity(id));
        }}
      />
    </nav>
  );
}
