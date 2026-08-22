import {
  AtSign,
  CheckCheck,
  Heart,
  type LucideIcon,
  Megaphone,
  Reply,
  ShieldAlert,
  UserPlus,
  X,
} from 'lucide-react';
import { useState } from 'react';
import { notifications } from '../data/notifications';
import { users } from '../data/users';
import { useUI } from '../store';
import type { NotificationKind } from '../types';
import { Avatar } from './ui/Avatar';
import { Portal } from './ui/Portal';

const kindMeta: Record<NotificationKind, { Icon: LucideIcon; color: string }> = {
  mention: { Icon: AtSign, color: '245 197 88' },
  reply: { Icon: Reply, color: '54 210 205' },
  reaction: { Icon: Heart, color: '240 97 168' },
  announcement: { Icon: Megaphone, color: '138 124 246' },
  friend: { Icon: UserPlus, color: '99 202 130' },
  system: { Icon: ShieldAlert, color: '99 179 237' },
};

export function NotificationPanel() {
  const open = useUI((s) => s.notifOpen);
  const setOpen = useUI((s) => s.setNotifOpen);
  const [tab, setTab] = useState<'all' | 'mentions'>('all');
  if (!open) return null;

  const list =
    tab === 'mentions' ? notifications.filter((n) => n.kind === 'mention') : notifications;

  return (
    <Portal>
      <div className="fixed inset-0 z-[700] animate-fade-in" onMouseDown={() => setOpen(false)}>
        <div
          className="absolute left-[80px] top-3 flex max-h-[80vh] w-[380px] flex-col overflow-hidden rounded-xl border border-line-2/70 bg-panel-2 shadow-pop animate-pop-in"
          onMouseDown={(e) => e.stopPropagation()}
        >
          <div className="flex items-center gap-2 border-b border-line/70 px-4 py-3">
            <h2 className="font-sans text-[14px] font-semibold text-ink">Inbox</h2>
            <div className="ml-2 flex rounded-md border border-line-2/50 bg-app p-0.5">
              {(['all', 'mentions'] as const).map((t) => (
                <button
                  key={t}
                  onClick={() => setTab(t)}
                  className={`rounded px-2.5 py-0.5 text-[12px] font-medium capitalize transition-colors ${
                    tab === t ? 'bg-surface-active text-ink' : 'text-ink-3 hover:text-ink-2'
                  }`}
                >
                  {t}
                </button>
              ))}
            </div>
            <button className="ml-auto flex items-center gap-1 rounded-md px-2 py-1 font-mono text-3xs text-ink-3 hover:bg-surface-hover hover:text-ink">
              <CheckCheck size={13} /> Read all
            </button>
            <button
              onClick={() => setOpen(false)}
              aria-label="Close inbox"
              className="grid h-7 w-7 place-items-center rounded-md text-ink-3 hover:bg-surface-hover hover:text-ink"
            >
              <X size={16} />
            </button>
          </div>

          <div className="min-h-0 flex-1 overflow-y-auto p-2">
            {list.map((n) => {
              const { Icon, color } = kindMeta[n.kind];
              const actor = n.actorId ? users[n.actorId] : null;
              return (
                <div
                  key={n.id}
                  className={`group relative flex gap-3 rounded-lg px-2.5 py-2.5 transition-colors hover:bg-surface-hover/60 ${
                    n.unread ? '' : 'opacity-70'
                  }`}
                >
                  {n.unread && (
                    <span className="absolute left-0 top-1/2 h-6 w-0.5 -translate-y-1/2 rounded-r bg-accent" />
                  )}
                  <div className="relative shrink-0">
                    {actor ? (
                      <Avatar user={actor} size={34} rounded="rounded-[32%]" />
                    ) : (
                      <span
                        className="grid h-[34px] w-[34px] place-items-center rounded-[32%] border border-line-2/50"
                        style={{ color: `rgb(${color})`, background: `rgb(${color} / 0.12)` }}
                      >
                        <Icon size={16} />
                      </span>
                    )}
                    <span
                      className="absolute -bottom-1 -right-1 grid h-4 w-4 place-items-center rounded-full border-2 border-panel-2"
                      style={{ color: `rgb(${color})`, background: 'rgb(var(--surface))' }}
                    >
                      <Icon size={9} strokeWidth={2.4} />
                    </span>
                  </div>
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span className="truncate text-[13px] font-semibold text-ink">{n.title}</span>
                      <span className="ml-auto shrink-0 font-mono text-3xs text-ink-4">{n.ts}</span>
                    </div>
                    <p className="mt-0.5 line-clamp-2 text-[12.5px] leading-snug text-ink-2">
                      {n.body}
                    </p>
                    {n.kind === 'friend' && (
                      <div className="mt-1.5 flex gap-2">
                        <button className="rounded-md bg-accent/90 px-2.5 py-1 text-[12px] font-semibold text-app hover:bg-accent">
                          Accept
                        </button>
                        <button className="rounded-md border border-line-2/60 px-2.5 py-1 text-[12px] font-medium text-ink-2 hover:bg-surface-hover">
                          Ignore
                        </button>
                      </div>
                    )}
                  </div>
                  {n.communityTag && (
                    <span className="chip absolute right-2 top-2 opacity-0 group-hover:opacity-100">
                      {n.communityTag}
                    </span>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </Portal>
  );
}
