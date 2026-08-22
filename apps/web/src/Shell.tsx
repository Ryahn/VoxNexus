import { useEffect } from 'react';
import { ChannelSidebar } from './components/ChannelSidebar';
import { ChatArea } from './components/ChatArea';
import { CommunityRail } from './components/CommunityRail';
import { ContextMenu } from './components/ContextMenu';
import { HomeView } from './components/HomeView';
import { MemberSidebar } from './components/MemberSidebar';
import { NotificationPanel } from './components/NotificationPanel';
import { SearchPalette } from './components/SearchPalette';
import { SettingsModal } from './components/SettingsModal';
import { ThreadPanel } from './components/ThreadPanel';
import { UserProfilePopover } from './components/UserProfilePopover';
import { useViewport } from './lib/useViewport';
import { useUI } from './store';

export function Shell() {
  useViewport();
  const activeCommunity = useUI((s) => s.activeCommunity);
  const navCollapsed = useUI((s) => s.navCollapsed);
  const membersOpen = useUI((s) => s.membersOpen);
  const threadOpen = useUI((s) => s.threadOpen);
  const setSearchOpen = useUI((s) => s.setSearchOpen);
  const searchOpen = useUI((s) => s.searchOpen);
  const setNotifOpen = useUI((s) => s.setNotifOpen);
  const notifOpen = useUI((s) => s.notifOpen);
  const setSettingsOpen = useUI((s) => s.setSettingsOpen);
  const settingsOpen = useUI((s) => s.settingsOpen);
  const toggleMembers = useUI((s) => s.toggleMembers);

  // global keyboard shortcuts
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const mod = e.ctrlKey || e.metaKey;
      if (!mod) return;
      const k = e.key.toLowerCase();
      if (k === 'k') {
        e.preventDefault();
        setSearchOpen(!searchOpen);
      } else if (k === 'b') {
        e.preventDefault();
        setNotifOpen(!notifOpen);
      } else if (k === 'u') {
        e.preventDefault();
        toggleMembers();
      } else if (e.key === ',') {
        e.preventDefault();
        setSettingsOpen(!settingsOpen);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [
    searchOpen,
    notifOpen,
    settingsOpen,
    setSearchOpen,
    setNotifOpen,
    setSettingsOpen,
    toggleMembers,
  ]);

  // give the thread panel room on smaller screens by retracting the nav
  useEffect(() => {
    if (threadOpen && window.innerWidth < 1080) useUI.setState({ navCollapsed: true });
    if (!threadOpen && window.innerWidth >= 680) useUI.setState({ navCollapsed: false });
  }, [threadOpen]);

  const isHome = activeCommunity === 'home';

  return (
    <div className="flex h-full w-full overflow-hidden bg-app text-ink">
      <CommunityRail />

      {isHome ? (
        <HomeView />
      ) : (
        <>
          {/* channel/group nav — collapsible */}
          <div
            className={`shrink-0 overflow-hidden transition-[width] duration-200 ease-swift ${
              navCollapsed ? 'w-0' : 'w-60'
            }`}
          >
            <ChannelSidebar />
          </div>

          <ChatArea />

          {membersOpen && !threadOpen && <MemberSidebar />}
          {threadOpen && <ThreadPanel />}
        </>
      )}

      {/* overlays */}
      <SearchPalette />
      <NotificationPanel />
      <ContextMenu />
      <UserProfilePopover />
      <SettingsModal />
    </div>
  );
}
