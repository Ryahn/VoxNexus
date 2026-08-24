import { create } from 'zustand';
import { DEFAULT_CHANNEL, DEFAULT_GROUP } from './data/structure';

export interface ContextMenuItem {
  label?: string;
  icon?: string;
  danger?: boolean;
  separator?: boolean;
  shortcut?: string;
  disabled?: boolean;
  onSelect?: () => void;
}

export interface Anchor {
  x: number;
  y: number;
  /** prefer opening to the left of x when true */
  left?: boolean;
  /** vertical align to bottom edge */
  bottom?: boolean;
}

export type PermissionOverrideTarget =
  | {
      scope: 'channel';
      communityId: string;
      channelId: string;
      name: string;
      categoryId: string | null;
    }
  | {
      scope: 'category';
      communityId: string;
      categoryId: string;
      name: string;
    };

interface UIState {
  // navigation
  activeCommunity: string;
  activeGroup: string;
  activeChannel: string;
  setCommunity: (id: string) => void;
  setGroup: (id: string) => void;
  setChannel: (id: string) => void;

  // panels
  navCollapsed: boolean;
  membersOpen: boolean;
  threadOpen: boolean;
  compact: boolean;
  toggleNav: () => void;
  toggleMembers: () => void;
  setThreadOpen: (v: boolean) => void;
  toggleCompact: () => void;

  // collapsible groups
  collapsedCats: Record<string, boolean>;
  toggleCat: (id: string) => void;
  collapsedSections: Record<string, boolean>;
  toggleSection: (id: string) => void;

  // voice
  voiceChannel: string | null;
  muted: boolean;
  deafened: boolean;
  video: boolean;
  screenshare: boolean;
  connectVoice: (id: string) => void;
  disconnectVoice: () => void;
  toggleMute: () => void;
  toggleDeafen: () => void;
  toggleVideo: () => void;
  toggleScreenshare: () => void;

  // overlays
  searchOpen: boolean;
  setSearchOpen: (v: boolean) => void;
  notifOpen: boolean;
  setNotifOpen: (v: boolean) => void;

  profile: { userId: string; anchor: Anchor } | null;
  openProfile: (userId: string, anchor: Anchor) => void;
  closeProfile: () => void;

  menu: { anchor: Anchor; items: ContextMenuItem[] } | null;
  openMenu: (anchor: Anchor, items: ContextMenuItem[]) => void;
  closeMenu: () => void;

  // message being replied to (composer state)
  replyingTo: string | null;
  setReplyingTo: (id: string | null) => void;

  // home / direct messages
  activeDM: string | null;
  setActiveDM: (id: string | null) => void;

  // settings
  settingsOpen: boolean;
  setSettingsOpen: (v: boolean) => void;

  // create community
  createCommunityOpen: boolean;
  setCreateCommunityOpen: (v: boolean) => void;

  // join community by id
  joinCommunityOpen: boolean;
  setJoinCommunityOpen: (v: boolean) => void;

  // invite manager
  inviteManagerOpen: boolean;
  setInviteManagerOpen: (v: boolean) => void;

  // community settings (join mode, etc.)
  communitySettingsOpen: boolean;
  setCommunitySettingsOpen: (v: boolean) => void;

  // permission overrides (F030)
  permissionOverrides: PermissionOverrideTarget | null;
  openPermissionOverrides: (target: PermissionOverrideTarget) => void;
  closePermissionOverrides: () => void;
}

export const useUI = create<UIState>((set) => ({
  activeCommunity: 'nexus',
  activeGroup: DEFAULT_GROUP,
  activeChannel: DEFAULT_CHANNEL,
  setCommunity: (id) => set({ activeCommunity: id }),
  setGroup: (id) => set({ activeGroup: id }),
  setChannel: (id) => set({ activeChannel: id, threadOpen: false }),

  navCollapsed: false,
  membersOpen: true,
  threadOpen: false,
  compact: false,
  toggleNav: () => set((s) => ({ navCollapsed: !s.navCollapsed })),
  toggleMembers: () => set((s) => ({ membersOpen: !s.membersOpen })),
  setThreadOpen: (v) => set({ threadOpen: v }),
  toggleCompact: () => set((s) => ({ compact: !s.compact })),

  collapsedCats: {},
  toggleCat: (id) =>
    set((s) => ({ collapsedCats: { ...s.collapsedCats, [id]: !s.collapsedCats[id] } })),
  collapsedSections: {},
  toggleSection: (id) =>
    set((s) => ({ collapsedSections: { ...s.collapsedSections, [id]: !s.collapsedSections[id] } })),

  voiceChannel: null,
  muted: false,
  deafened: false,
  video: false,
  screenshare: false,
  connectVoice: (id) => set({ voiceChannel: id }),
  disconnectVoice: () => set({ voiceChannel: null, video: false, screenshare: false }),
  toggleMute: () => set((s) => ({ muted: !s.muted })),
  toggleDeafen: () =>
    set((s) => {
      const deafened = !s.deafened;
      return { deafened, muted: deafened ? true : s.muted };
    }),
  toggleVideo: () => set((s) => ({ video: !s.video })),
  toggleScreenshare: () => set((s) => ({ screenshare: !s.screenshare })),

  searchOpen: false,
  setSearchOpen: (v) => set({ searchOpen: v }),
  notifOpen: false,
  setNotifOpen: (v) => set({ notifOpen: v }),

  profile: null,
  openProfile: (userId, anchor) => set({ profile: { userId, anchor }, menu: null }),
  closeProfile: () => set({ profile: null }),

  menu: null,
  openMenu: (anchor, items) => set({ menu: { anchor, items }, profile: null }),
  closeMenu: () => set({ menu: null }),

  replyingTo: null,
  setReplyingTo: (id) => set({ replyingTo: id }),

  activeDM: null,
  setActiveDM: (id) => set({ activeDM: id }),

  settingsOpen: false,
  setSettingsOpen: (v) => set({ settingsOpen: v }),

  createCommunityOpen: false,
  setCreateCommunityOpen: (v) => set({ createCommunityOpen: v }),

  joinCommunityOpen: false,
  setJoinCommunityOpen: (v) => set({ joinCommunityOpen: v }),

  inviteManagerOpen: false,
  setInviteManagerOpen: (v) => set({ inviteManagerOpen: v }),

  communitySettingsOpen: false,
  setCommunitySettingsOpen: (v) => set({ communitySettingsOpen: v }),

  permissionOverrides: null,
  openPermissionOverrides: (target) => set({ permissionOverrides: target }),
  closePermissionOverrides: () => set({ permissionOverrides: null }),
}));
