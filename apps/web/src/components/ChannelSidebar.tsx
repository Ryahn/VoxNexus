import { categories, channels, groups } from '../data/structure';
import { useUI } from '../store';
import { ChannelCategory } from './ChannelCategory';
import { CommunityHeader } from './CommunityHeader';
import { GroupSelector } from './GroupSelector';
import { UserControlBar } from './UserControlBar';
import { VoicePanel } from './VoicePanel';

export function ChannelSidebar() {
  const activeGroup = useUI((s) => s.activeGroup);
  const voiceChannel = useUI((s) => s.voiceChannel);
  const group = groups.find((g) => g.id === activeGroup);

  const groupCats = categories.filter((c) => c.groupId === activeGroup);

  return (
    <aside className="flex h-full w-60 shrink-0 flex-col border-r border-line/70 bg-panel">
      <CommunityHeader />
      <GroupSelector />

      {/* Channels stay mock until F027; label falls back when live Space is selected. */}
      <div className="flex items-center justify-between px-3 pb-1 pt-2">
        <span className="font-sans text-[11px] font-semibold uppercase tracking-[0.12em] text-ink-3">
          {group?.name ?? 'Channels'}
        </span>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-3">
        {groupCats.length === 0 ? (
          <p className="px-2 py-2 text-[12px] text-ink-4">Channels arrive in a later milestone.</p>
        ) : (
          groupCats.map((cat) => (
            <ChannelCategory
              key={cat.id}
              category={cat}
              channels={channels.filter((c) => c.categoryId === cat.id)}
            />
          ))
        )}
      </div>

      {voiceChannel && <VoicePanel />}
      <UserControlBar />
    </aside>
  );
}
