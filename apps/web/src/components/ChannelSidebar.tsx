import { channels, groups, categories as mockCategories } from '../data/structure';
import { useUI } from '../store';
import { ChannelCategory } from './ChannelCategory';
import { CommunityHeader } from './CommunityHeader';
import { GroupSelector } from './GroupSelector';
import { LiveCategoryList } from './LiveCategoryList';
import { UserControlBar } from './UserControlBar';
import { VoicePanel } from './VoicePanel';

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export function ChannelSidebar() {
  const activeCommunity = useUI((s) => s.activeCommunity);
  const activeGroup = useUI((s) => s.activeGroup);
  const voiceChannel = useUI((s) => s.voiceChannel);
  const group = groups.find((g) => g.id === activeGroup);
  const isLiveSpace = UUID_RE.test(activeCommunity) && UUID_RE.test(activeGroup);
  const groupCats = mockCategories.filter((c) => c.groupId === activeGroup);

  return (
    <aside className="flex h-full w-60 shrink-0 flex-col border-r border-line/70 bg-panel">
      <CommunityHeader />
      <GroupSelector />

      {isLiveSpace ? (
        <LiveCategoryList
          communityId={activeCommunity}
          spaceId={activeGroup}
          spaceName={group?.name ?? 'Channels'}
        />
      ) : (
        <>
          <div className="flex items-center justify-between px-3 pb-1 pt-2">
            <span className="font-sans text-[11px] font-semibold uppercase tracking-[0.12em] text-ink-3">
              {group?.name ?? 'Channels'}
            </span>
          </div>
          <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-3">
            {groupCats.length === 0 ? (
              <p className="px-2 py-2 text-[12px] text-ink-4">
                Channels arrive in a later milestone.
              </p>
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
        </>
      )}

      {voiceChannel && <VoicePanel />}
      <UserControlBar />
    </aside>
  );
}
