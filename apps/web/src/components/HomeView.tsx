import { Compass, MessageSquarePlus, Search, Sparkles, Users2 } from 'lucide-react';
import { dmList } from '../data/dms';
import { users } from '../data/users';
import { useUI } from '../store';
import { DMChatView } from './DMChatView';
import { UserControlBar } from './UserControlBar';
import { Avatar } from './ui/Avatar';

export function HomeView() {
  const activeDM = useUI((s) => s.activeDM);
  const setActiveDM = useUI((s) => s.setActiveDM);
  const setCommunity = useUI((s) => s.setCommunity);

  return (
    <div className="flex min-w-0 flex-1">
      {/* DM sidebar */}
      <aside className="flex h-full w-60 shrink-0 flex-col border-r border-line/70 bg-panel">
        <div className="border-b border-line/70 p-2">
          <button className="flex w-full items-center gap-2 rounded-md border border-line-2/50 bg-input px-2.5 py-1.5 text-ink-3 transition-colors hover:border-accent/40 hover:text-ink-2">
            <Search size={14} />
            <span className="flex-1 text-left text-xs">Find or start a conversation</span>
          </button>
        </div>

        <div className="min-h-0 flex-1 overflow-y-auto px-2 py-2">
          <button
            onClick={() => setActiveDM(null)}
            className={`mb-1 flex w-full items-center gap-2.5 rounded-md px-2.5 py-2 text-left text-[13px] font-medium transition-colors ${
              activeDM === null
                ? 'bg-surface-active text-ink'
                : 'text-ink-2 hover:bg-surface-hover/70 hover:text-ink'
            }`}
          >
            <Users2 size={17} className={activeDM === null ? 'text-accent' : 'text-ink-3'} />{' '}
            Friends
          </button>

          <div className="flex items-center justify-between px-2 pb-1 pt-2">
            <span className="kicker">Direct Messages</span>
            <MessageSquarePlus size={13} className="cursor-pointer text-ink-4 hover:text-ink-2" />
          </div>

          <div className="flex flex-col gap-px">
            {dmList.map((dm) => {
              const u = users[dm.userId];
              if (!u) return null;
              const active = activeDM === dm.userId;
              return (
                <button
                  key={dm.userId}
                  onClick={() => setActiveDM(dm.userId)}
                  className={`group relative flex w-full items-center gap-2.5 rounded-md px-2 py-1.5 text-left transition-colors ${
                    active ? 'bg-surface-active' : 'hover:bg-surface-hover/70'
                  }`}
                >
                  {active && <span className="tick" />}
                  <Avatar
                    user={u}
                    size={32}
                    rounded="rounded-full"
                    showPresence
                    ring="rgb(var(--bg-panel))"
                  />
                  <span className="min-w-0 flex-1 leading-tight">
                    <span
                      className={`block truncate text-[13px] font-medium ${active ? 'text-ink' : 'text-ink-2 group-hover:text-ink'}`}
                    >
                      {u.displayName}
                    </span>
                    <span className="block truncate font-mono text-3xs text-ink-3">
                      {dm.preview}
                    </span>
                  </span>
                  {dm.unread ? (
                    <span className="grid h-4 min-w-[16px] shrink-0 place-items-center rounded-full bg-[rgb(var(--mention))] px-1 font-mono text-3xs font-bold text-app">
                      {dm.unread}
                    </span>
                  ) : (
                    <span className="shrink-0 font-mono text-3xs text-ink-4">{dm.time}</span>
                  )}
                </button>
              );
            })}
          </div>
        </div>

        <UserControlBar />
      </aside>

      {/* main region */}
      {activeDM ? (
        <DMChatView userId={activeDM} />
      ) : (
        <div className="relative flex flex-1 flex-col items-center justify-center overflow-hidden bg-app px-6 text-center">
          <div
            aria-hidden
            className="pointer-events-none absolute inset-0 grid-veil opacity-[0.35]"
          />
          <div
            aria-hidden
            className="pointer-events-none absolute left-1/2 top-1/3 h-[420px] w-[420px] -translate-x-1/2 rounded-full opacity-30 blur-3xl"
            style={{
              background: 'radial-gradient(circle, rgb(var(--accent) / 0.4), transparent 60%)',
            }}
          />
          <div className="relative z-10 max-w-md">
            <div className="mx-auto mb-5 grid h-16 w-16 place-items-center rounded-2xl border border-accent/30 bg-accent/10">
              <Sparkles size={30} className="text-accent" />
            </div>
            <h1 className="font-sans text-[26px] font-bold tracking-tight text-ink">
              Welcome back to VOX
            </h1>
            <p className="mx-auto mt-2 max-w-sm text-[14px] leading-relaxed text-ink-2">
              Pick up a conversation, or dive into a community. Your workspaces stay in sync across
              every device.
            </p>
            <div className="mt-6 flex items-center justify-center gap-2.5">
              <button
                onClick={() => setActiveDM(dmList[0].userId)}
                className="flex items-center gap-2 rounded-lg bg-accent/90 px-4 py-2 text-[13px] font-semibold text-app hover:bg-accent"
              >
                <MessageSquarePlus size={16} /> Open a Message
              </button>
              <button
                onClick={() => setCommunity('nexus')}
                className="flex items-center gap-2 rounded-lg border border-line-2/60 px-4 py-2 text-[13px] font-medium text-ink-2 hover:bg-surface-hover hover:text-ink"
              >
                <Compass size={16} /> Open Project Nexus
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
