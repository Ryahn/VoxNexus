import { BellOff, Lock } from 'lucide-react';
import { users } from '../data/users';
import { channelMeta } from '../lib/channelMeta';
import { menuFor } from '../lib/menus';
import { useUI } from '../store';
import type { Channel } from '../types';
import { Avatar } from './ui/Avatar';

type Props = {
  channel: Channel;
  communityId?: string;
  canManage?: boolean;
};

export function ChannelItem({ channel, communityId, canManage }: Props) {
  const active = useUI((s) => s.activeChannel);
  const setChannel = useUI((s) => s.setChannel);
  const connectVoice = useUI((s) => s.connectVoice);
  const openMenu = useUI((s) => s.openMenu);
  const openPermissionOverrides = useUI((s) => s.openPermissionOverrides);
  const { Icon } = channelMeta[channel.type];

  const isActive = active === channel.id;
  const isVoice = channel.type === 'voice';
  const unread = channel.unread && !isActive;
  const mentions = channel.mentions ?? 0;

  const select = () => {
    if (isVoice) connectVoice(channel.id);
    else setChannel(channel.id);
  };

  const menuItems = menuFor('channel', channel.name).map((item) => {
    if (item.label === 'Edit Permissions' && communityId && canManage) {
      return {
        ...item,
        onSelect: () =>
          openPermissionOverrides({
            scope: 'channel',
            communityId,
            channelId: channel.id,
            name: channel.name,
            categoryId: channel.categoryId ?? null,
          }),
      };
    }
    return item;
  });

  return (
    <div className="flex flex-col">
      <button
        type="button"
        onClick={select}
        onContextMenu={(e) => {
          e.preventDefault();
          openMenu({ x: e.clientX, y: e.clientY }, menuItems);
        }}
        aria-current={isActive}
        className={`group relative flex items-center gap-1.5 rounded-md px-2 py-[5px] text-left transition-colors duration-150 ${
          isActive
            ? 'bg-surface-active text-ink'
            : unread
              ? 'text-ink hover:bg-surface-hover/60'
              : 'text-ink-3 hover:bg-surface-hover/60 hover:text-ink-2'
        } ${channel.muted ? 'opacity-55' : ''}`}
      >
        {isActive && <span className="tick" />}
        {/* unread dot to the left of muted-eligible items */}
        {unread && !isActive && (
          <span className="absolute -left-1.5 h-1.5 w-1.5 rounded-full bg-ink" aria-hidden />
        )}
        <Icon
          size={16}
          strokeWidth={1.9}
          className={`shrink-0 ${isActive ? 'text-accent' : unread ? 'text-ink-2' : 'text-ink-4 group-hover:text-ink-3'}`}
        />
        <span
          className={`flex-1 truncate text-[13px] ${unread || isActive ? 'font-semibold' : 'font-medium'}`}
        >
          {channel.name}
        </span>

        {channel.locked && <Lock size={12} className="shrink-0 text-ink-4" />}
        {channel.muted && <BellOff size={12} className="shrink-0 text-ink-4" />}
        {isVoice && (channel.liveCount ?? 0) > 0 && (
          <span className="chip !border-online/30 !text-online">
            <span className="h-1 w-1 rounded-full bg-online" />
            {channel.liveCount}
          </span>
        )}
        {mentions > 0 && (
          <span className="grid h-4 min-w-[16px] place-items-center rounded-full bg-[rgb(var(--mention))] px-1 font-mono text-3xs font-bold text-app">
            {mentions}
          </span>
        )}
      </button>

      {/* Voice: connected members list, indented */}
      {isVoice && channel.connected && channel.connected.length > 0 && (
        <div className="ml-6 flex flex-col gap-0.5 border-l border-line-2/50 pb-1 pl-2 pt-0.5">
          {channel.connected.map((uid) => {
            const u = users[uid];
            if (!u) return null;
            return (
              <div
                key={uid}
                className="flex items-center gap-1.5 rounded px-1 py-0.5 text-ink-2 hover:bg-surface-hover/50"
              >
                <Avatar user={u} size={18} rounded="rounded-full" />
                <span className="truncate text-xs font-medium">{u.displayName}</span>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
