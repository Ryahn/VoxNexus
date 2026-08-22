import { Code2, Gamepad2, type LucideIcon, Music, Radio } from 'lucide-react';
import { nameColor } from '../data/roles';
import { memberMenu } from '../lib/menus';
import { useUI } from '../store';
import type { User } from '../types';
import { Avatar } from './ui/Avatar';

const activityIcon: Record<string, LucideIcon> = {
  coding: Code2,
  listening: Music,
  streaming: Radio,
  playing: Gamepad2,
};

export function MemberItem({ user }: { user: User }) {
  const openProfile = useUI((s) => s.openProfile);
  const openMenu = useUI((s) => s.openMenu);
  const color = nameColor(user.roleIds);
  const offline = user.presence === 'offline';
  const AIcon = user.activity ? activityIcon[user.activity.kind] : null;

  return (
    <button
      type="button"
      onClick={(e) => {
        const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
        openProfile(user.id, { x: r.left, y: r.top, left: true });
      }}
      onContextMenu={(e) => {
        e.preventDefault();
        openMenu({ x: e.clientX, y: e.clientY, left: true }, memberMenu(user.displayName));
      }}
      className={`group flex w-full items-center gap-2.5 rounded-md px-2 py-1 text-left transition-colors hover:bg-surface-hover/70 ${
        offline ? 'opacity-45 hover:opacity-80' : ''
      }`}
    >
      <Avatar
        user={user}
        size={30}
        rounded="rounded-full"
        showPresence
        ring="rgb(var(--bg-panel))"
        dim={offline}
      />
      <span className="min-w-0 flex-1 leading-tight">
        <span className="flex items-center gap-1.5">
          <span
            className="truncate text-[13px] font-medium"
            style={{ color: offline ? 'rgb(var(--text-2))' : `rgb(${color})` }}
          >
            {user.displayName}
          </span>
        </span>
        {user.activity ? (
          <span className="flex items-center gap-1 truncate font-mono text-3xs text-ink-3">
            {AIcon && <AIcon size={9} className="shrink-0" />}
            <span className="truncate">{user.activity.label}</span>
          </span>
        ) : (
          user.status &&
          !offline && <span className="block truncate text-3xs text-ink-3">{user.status}</span>
        )}
      </span>
    </button>
  );
}
