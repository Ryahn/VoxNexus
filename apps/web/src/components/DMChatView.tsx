import { MoreVertical, Phone, Pin, UserPlus, Video } from 'lucide-react';
import { useEffect, useRef } from 'react';
import { dmConversations } from '../data/dms';
import { users } from '../data/users';
import { useUI } from '../store';
import { Message } from './Message';
import { MessageComposer } from './MessageComposer';
import { Avatar } from './ui/Avatar';
import { IconButton } from './ui/IconButton';
import { PresenceDot } from './ui/Presence';

const presenceLabel: Record<string, string> = {
  online: 'Online',
  idle: 'Idle',
  dnd: 'Do Not Disturb',
  offline: 'Offline',
};

export function DMChatView({ userId }: { userId: string }) {
  const convo = dmConversations[userId];
  const user = users[userId];
  const openProfile = useUI((s) => s.openProfile);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView();
  }, [userId]);

  if (!convo || !user) return null;

  return (
    <main className="relative flex min-w-0 flex-1 flex-col bg-app">
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 opacity-50"
        style={{
          background:
            'radial-gradient(90% 60% at 80% -10%, rgb(var(--accent) / 0.05), transparent 55%), radial-gradient(70% 50% at 0% 110%, rgb(var(--accent-2) / 0.05), transparent 60%)',
        }}
      />
      <div className="relative flex min-h-0 flex-1 flex-col">
        {/* header */}
        <header className="flex h-12 shrink-0 items-center gap-2.5 border-b border-line/70 bg-app px-3">
          <button
            type="button"
            onClick={(e) => {
              const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
              openProfile(user.id, { x: r.left, y: r.bottom + 6 });
            }}
            className="flex min-w-0 items-center gap-2.5"
          >
            <Avatar
              user={user}
              size={26}
              rounded="rounded-full"
              showPresence
              ring="rgb(var(--bg-app))"
            />
            <span className="truncate font-sans text-[14px] font-semibold text-ink">
              {user.displayName}
            </span>
            <span className="hidden items-center gap-1 sm:flex">
              <PresenceDot presence={user.presence} size={7} ring="rgb(var(--bg-app))" />
              <span className="font-mono text-3xs text-ink-3">{presenceLabel[user.presence]}</span>
            </span>
          </button>
          <div className="ml-auto flex items-center gap-0.5">
            <IconButton icon={Phone} label="Start Voice Call" />
            <IconButton icon={Video} label="Start Video Call" />
            <span className="hidden md:flex">
              <IconButton icon={Pin} label="Pinned Messages" />
            </span>
            <span className="hidden md:flex">
              <IconButton icon={UserPlus} label="Add to Group" />
            </span>
            <IconButton icon={MoreVertical} label="More" />
          </div>
        </header>

        {/* messages */}
        <div className="min-h-0 flex-1 overflow-y-auto overflow-x-hidden">
          {/* intro */}
          <div className="px-4 pb-2 pt-6">
            <Avatar
              user={user}
              size={72}
              rounded="rounded-[32%]"
              showPresence
              ring="rgb(var(--bg-app))"
            />
            <h2 className="mt-3 font-sans text-[22px] font-bold text-ink">{user.displayName}</h2>
            <p className="font-mono text-[12px] text-ink-3">@{user.username}</p>
            <p className="mt-2 max-w-lg text-[13.5px] leading-relaxed text-ink-2">
              This is the beginning of your direct message history with{' '}
              <span className="text-ink">{user.displayName}</span>.{user.bio ? ` ${user.bio}` : ''}
            </p>
            <div className="mt-3 flex items-center gap-2">
              <span className="chip">{user.mutuals ?? 0} mutual communities</span>
              <span className="chip">Member since {user.memberSince ?? '—'}</span>
            </div>
          </div>

          <div className="mb-2 flex items-center gap-3 px-4">
            <div className="h-px flex-1 bg-line/70" />
            <span className="rounded-full border border-line-2/50 bg-panel px-2.5 py-0.5 font-mono text-3xs uppercase tracking-wider text-ink-3">
              Today
            </span>
            <div className="h-px flex-1 bg-line/70" />
          </div>

          <div className="pb-3">
            {convo.messages.map((msg, i) => {
              const prev = convo.messages[i - 1];
              const grouped = Boolean(prev && prev.authorId === msg.authorId && !msg.replyTo);
              return <Message key={msg.id} message={msg} grouped={grouped} />;
            })}
          </div>
          <div ref={bottomRef} />
        </div>

        <MessageComposer placeholder={`Message @${user.username}`} />
      </div>
    </main>
  );
}
