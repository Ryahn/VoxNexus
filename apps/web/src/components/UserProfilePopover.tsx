import { MessageSquare, MoreHorizontal, Phone, UserPlus } from 'lucide-react';
import { roles } from '../data/roles';
import { users } from '../data/users';
import { bannerGradient } from '../lib/avatar';
import { useUI } from '../store';
import { Avatar } from './ui/Avatar';
import { Popover } from './ui/Popover';
import { PresenceDot } from './ui/Presence';

const presenceLabel: Record<string, string> = {
  online: 'Online',
  idle: 'Idle',
  dnd: 'Do Not Disturb',
  offline: 'Offline',
};

export function UserProfilePopover() {
  const profile = useUI((s) => s.profile);
  const close = useUI((s) => s.closeProfile);
  if (!profile) return null;
  const user = users[profile.userId];
  if (!user) return null;
  const userRoles = user.roleIds.map((id) => roles[id]).filter(Boolean);

  return (
    <Popover anchor={profile.anchor} onClose={close} width={320}>
      <div className="overflow-hidden rounded-xl border border-line-2/70 bg-panel-2 shadow-pop">
        {/* accent hairline top */}
        <div
          className="h-[2px] w-full"
          style={{
            background: `linear-gradient(90deg, transparent, rgb(${user.accent}), transparent)`,
          }}
        />

        {/* banner */}
        <div
          className="relative h-20"
          style={{ background: bannerGradient(user.bannerSeed ?? user.avatarSeed) }}
        >
          <div className="absolute inset-0 grid-veil opacity-30" />
          {/* angular clipped corner detail */}
          <div className="absolute right-3 top-2 flex items-center gap-1 font-mono text-3xs uppercase tracking-widest text-white/70">
            <span className="h-1 w-1 rounded-full bg-white/80" /> NX-ID · {user.id.toUpperCase()}
          </div>
        </div>

        <div className="px-4 pb-4">
          {/* avatar overlapping banner */}
          <div className="-mt-9 mb-2 flex items-end justify-between">
            <div className="rounded-[34%] border-[3px] border-panel-2 bg-panel-2">
              <Avatar
                user={user}
                size={64}
                rounded="rounded-[30%]"
                showPresence
                ring="rgb(var(--bg-panel-2))"
              />
            </div>
            <div className="mb-1 flex items-center gap-1">
              <button className="grid h-8 w-8 place-items-center rounded-md border border-line-2/60 bg-surface/70 text-ink-2 hover:text-accent">
                <Phone size={15} />
              </button>
              <button className="grid h-8 w-8 place-items-center rounded-md border border-line-2/60 bg-surface/70 text-ink-2 hover:text-accent">
                <UserPlus size={15} />
              </button>
              <button className="grid h-8 w-8 place-items-center rounded-md border border-line-2/60 bg-surface/70 text-ink-2 hover:text-ink">
                <MoreHorizontal size={16} />
              </button>
            </div>
          </div>

          <div className="rounded-lg border border-line-2/50 bg-app/60 p-3">
            <div className="flex items-center gap-2">
              <h3 className="font-sans text-[17px] font-bold text-ink">{user.displayName}</h3>
              {user.pronouns && (
                <span className="font-mono text-3xs text-ink-3">· {user.pronouns}</span>
              )}
            </div>
            <div className="font-mono text-[12px] text-ink-3">@{user.username}</div>

            <div className="mt-2 flex items-center gap-1.5">
              <PresenceDot presence={user.presence} size={9} ring="rgb(var(--input))" />
              <span className="text-[12px] text-ink-2">
                {user.status ?? presenceLabel[user.presence]}
              </span>
            </div>

            {user.bio && (
              <>
                <div className="my-2.5 h-px bg-line-2/50" />
                <div className="kicker mb-1">About</div>
                <p className="text-[12.5px] leading-relaxed text-ink-2">{user.bio}</p>
              </>
            )}

            {userRoles.length > 0 && (
              <>
                <div className="kicker mb-1.5 mt-3">Roles</div>
                <div className="flex flex-wrap gap-1.5">
                  {userRoles.map((r) => (
                    <span
                      key={r.id}
                      className="flex items-center gap-1.5 rounded-md border px-2 py-0.5 text-[11px] font-medium"
                      style={{
                        borderColor: `rgb(${r.color} / 0.4)`,
                        color: `rgb(${r.color})`,
                        background: `rgb(${r.color} / 0.08)`,
                      }}
                    >
                      <span
                        className="h-1.5 w-1.5 rounded-full"
                        style={{ background: `rgb(${r.color})` }}
                      />
                      {r.name}
                    </span>
                  ))}
                </div>
              </>
            )}

            <div className="mt-3 grid grid-cols-2 gap-2">
              <Stat label="Member Since" value={user.memberSince ?? '—'} />
              <Stat label="Mutual" value={`${user.mutuals ?? 0} communities`} />
            </div>
          </div>

          {user.id !== 'me' && (
            <button className="mt-3 flex w-full items-center justify-center gap-2 rounded-lg bg-accent/90 py-2 text-[13px] font-semibold text-app transition-colors hover:bg-accent">
              <MessageSquare size={15} /> Message {user.displayName}
            </button>
          )}
        </div>
      </div>
    </Popover>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border border-line-2/40 bg-surface/40 px-2 py-1.5">
      <div className="kicker">{label}</div>
      <div className="mt-0.5 text-[12px] font-medium text-ink">{value}</div>
    </div>
  );
}
