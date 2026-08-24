import {
  type CommunityMemberResponse,
  type CommunityResponse,
  deleteCommunity,
  getCommunity,
  type JoinMode,
  listCommunityMembers,
  transferCommunity,
  updateCommunity,
  uploadCommunityBanner,
  uploadCommunityIcon,
  uploadCommunityInviteSplash,
  uploadCommunityTagBadge,
} from '@voxnexus/api-client';
import {
  type LucideIcon,
  ImagePlus,
  Shield,
  Settings2,
  Trash2,
  Users,
  X,
} from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { useAuth } from '../auth';
import { readApiErrorMessage } from '../lib/apiError';
import { useUI } from '../store';
import { CommunityRolesPanel } from './CommunityRolesPanel';
import { Portal } from './ui/Portal';

type Props = {
  communityId: string;
};

type Section = 'overview' | 'roles' | 'members' | 'danger';

const credentials = { credentials: 'include' as const };

type ImageKind = 'icon' | 'banner' | 'tagBadge' | 'inviteSplash';

function rgbStringToHex(color: string): string {
  if (color.startsWith('#')) return color.length === 7 ? color : '#36d2cd';
  const parts = color.trim().split(/\s+/).map(Number);
  if (parts.length !== 3 || parts.some((n) => Number.isNaN(n))) return '#36d2cd';
  return `#${parts
    .map((n) => Math.max(0, Math.min(255, Math.round(n))).toString(16).padStart(2, '0'))
    .join('')}`;
}

function hexToRgbString(hex: string): string {
  const h = hex.replace('#', '');
  if (h.length !== 6) return '54 210 205';
  const r = parseInt(h.slice(0, 2), 16);
  const g = parseInt(h.slice(2, 4), 16);
  const b = parseInt(h.slice(4, 6), 16);
  if ([r, g, b].some((n) => Number.isNaN(n))) return '54 210 205';
  return `${r} ${g} ${b}`;
}

const nav: { id: Section; label: string; Icon: LucideIcon; group: string }[] = [
  { id: 'overview', label: 'Overview', Icon: Settings2, group: 'COMMUNITY' },
  { id: 'roles', label: 'Roles', Icon: Shield, group: 'COMMUNITY' },
  { id: 'members', label: 'Members', Icon: Users, group: 'COMMUNITY' },
  { id: 'danger', label: 'Danger Zone', Icon: Trash2, group: 'DANGER' },
];

export function CommunitySettingsModal({ communityId }: Props) {
  const { session } = useAuth();
  const open = useUI((s) => s.communitySettingsOpen);
  const setOpen = useUI((s) => s.setCommunitySettingsOpen);
  const setActiveCommunity = useUI((s) => s.setCommunity);
  const [community, setCommunity] = useState<CommunityResponse | null>(null);
  const [members, setMembers] = useState<CommunityMemberResponse[]>([]);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [joinMode, setJoinMode] = useState<Exclude<JoinMode, 'application'>>('open');
  const [discoverable, setDiscoverable] = useState(true);
  const [iconUrl, setIconUrl] = useState<string | null>(null);
  const [bannerUrl, setBannerUrl] = useState<string | null>(null);
  const [pendingIcon, setPendingIcon] = useState<File | null>(null);
  const [pendingBanner, setPendingBanner] = useState<File | null>(null);
  const [iconPreview, setIconPreview] = useState<string | null>(null);
  const [bannerPreview, setBannerPreview] = useState<string | null>(null);
  const [tagName, setTagName] = useState('');
  const [tagColor, setTagColor] = useState('54 210 205');
  const [invitePath, setInvitePath] = useState('');
  const [tagBadgeUrl, setTagBadgeUrl] = useState<string | null>(null);
  const [inviteSplashUrl, setInviteSplashUrl] = useState<string | null>(null);
  const [pendingTagBadge, setPendingTagBadge] = useState<File | null>(null);
  const [pendingInviteSplash, setPendingInviteSplash] = useState<File | null>(null);
  const [tagBadgePreview, setTagBadgePreview] = useState<string | null>(null);
  const [inviteSplashPreview, setInviteSplashPreview] = useState<string | null>(null);
  const [mediaBust, setMediaBust] = useState(0);
  const [transferTarget, setTransferTarget] = useState('');
  const [deleteConfirm, setDeleteConfirm] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [section, setSection] = useState<Section>('overview');
  const iconRef = useRef<HTMLInputElement>(null);
  const bannerRef = useRef<HTMLInputElement>(null);
  const tagBadgeRef = useRef<HTMLInputElement>(null);
  const inviteSplashRef = useRef<HTMLInputElement>(null);

  const isOwner = community?.owner_account_id === session.account.id;

  const refresh = async () => {
    const [communityResult, membersResult] = await Promise.all([
      getCommunity({ path: { community_id: communityId } }),
      listCommunityMembers({ path: { community_id: communityId }, query: { limit: 100 } }),
    ]);
    if (communityResult.error || !communityResult.data) {
      setError(readApiErrorMessage(communityResult.error, 'Could not load community settings.'));
      setCommunity(null);
      return;
    }
    const c = communityResult.data;
    setCommunity(c);
    setName(c.name);
    setDescription(c.description ?? '');
    setJoinMode(c.join_mode === 'invite' ? 'invite' : 'open');
    setDiscoverable(c.discoverable_on_instance);
    setIconUrl(c.icon_url ?? null);
    setBannerUrl(c.banner_url ?? null);
    setTagName(c.tag_name ?? '');
    setTagColor(c.tag_color?.trim() || '54 210 205');
    setInvitePath(c.invite_path ?? '');
    setTagBadgeUrl(c.tag_badge_url ?? null);
    setInviteSplashUrl(c.invite_splash_url ?? null);
    if (membersResult.data?.items) {
      setMembers(membersResult.data.items);
    }
  };

  useEffect(() => {
    if (!open) return;
    setError(null);
    setPending(false);
    setTransferTarget('');
    setDeleteConfirm('');
    setPendingIcon(null);
    setPendingBanner(null);
    setPendingTagBadge(null);
    setPendingInviteSplash(null);
    setIconPreview((prev) => {
      if (prev) URL.revokeObjectURL(prev);
      return null;
    });
    setBannerPreview((prev) => {
      if (prev) URL.revokeObjectURL(prev);
      return null;
    });
    setTagBadgePreview((prev) => {
      if (prev) URL.revokeObjectURL(prev);
      return null;
    });
    setInviteSplashPreview((prev) => {
      if (prev) URL.revokeObjectURL(prev);
      return null;
    });
    setSection('overview');
    void refresh();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setOpen(false);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, communityId, setOpen]);

  useEffect(() => {
    return () => {
      if (iconPreview) URL.revokeObjectURL(iconPreview);
    };
  }, [iconPreview]);

  useEffect(() => {
    return () => {
      if (bannerPreview) URL.revokeObjectURL(bannerPreview);
    };
  }, [bannerPreview]);

  useEffect(() => {
    return () => {
      if (tagBadgePreview) URL.revokeObjectURL(tagBadgePreview);
    };
  }, [tagBadgePreview]);

  useEffect(() => {
    return () => {
      if (inviteSplashPreview) URL.revokeObjectURL(inviteSplashPreview);
    };
  }, [inviteSplashPreview]);

  if (!open) return null;

  const pickImage = (kind: ImageKind, file: File | undefined) => {
    if (!file) return;
    const small = kind === 'icon' || kind === 'tagBadge';
    const maxBytes = small ? 2 * 1024 * 1024 : 5 * 1024 * 1024;
    if (file.size > maxBytes) {
      setError(
        small ? 'Image must be 2 MB or smaller.' : 'Image must be 5 MB or smaller.',
      );
      return;
    }
    if (!/^image\/(png|jpeg|gif|webp)$/.test(file.type)) {
      setError('Image must be JPEG, PNG, GIF, or WebP.');
      return;
    }
    const preview = URL.createObjectURL(file);
    if (kind === 'icon') {
      setIconPreview((previous) => {
        if (previous) URL.revokeObjectURL(previous);
        return preview;
      });
      setPendingIcon(file);
    } else if (kind === 'banner') {
      setBannerPreview((previous) => {
        if (previous) URL.revokeObjectURL(previous);
        return preview;
      });
      setPendingBanner(file);
    } else if (kind === 'tagBadge') {
      setTagBadgePreview((previous) => {
        if (previous) URL.revokeObjectURL(previous);
        return preview;
      });
      setPendingTagBadge(file);
    } else {
      setInviteSplashPreview((previous) => {
        if (previous) URL.revokeObjectURL(previous);
        return preview;
      });
      setPendingInviteSplash(file);
    }
    setError(null);
  };

  const uploadPending = async (
    kind: ImageKind,
    file: File,
  ): Promise<{ ok: true; data: CommunityResponse } | { ok: false; message: string }> => {
    const bytes = new Uint8Array(await file.arrayBuffer());
    const common = {
      path: { community_id: communityId },
      body: bytes,
      headers: { 'Content-Type': 'application/octet-stream' },
      ...credentials,
    };
    const result =
      kind === 'icon'
        ? await uploadCommunityIcon(common)
        : kind === 'banner'
          ? await uploadCommunityBanner(common)
          : kind === 'tagBadge'
            ? await uploadCommunityTagBadge(common)
            : await uploadCommunityInviteSplash(common);
    if (result.error || !result.data) {
      return {
        ok: false,
        message: readApiErrorMessage(result.error, 'Upload failed (check size and image type).'),
      };
    }
    return { ok: true, data: result.data };
  };

  const applyCommunityMedia = (data: CommunityResponse) => {
    setIconUrl(data.icon_url ?? null);
    setBannerUrl(data.banner_url ?? null);
    setTagBadgeUrl(data.tag_badge_url ?? null);
    setInviteSplashUrl(data.invite_splash_url ?? null);
  };

  const save = async () => {
    if (!isOwner) return;
    const trimmed = name.trim();
    if (!trimmed) {
      setError('Community name is required.');
      return;
    }
    setPending(true);
    setError(null);
    try {
      for (const [kind, file] of [
        ['icon', pendingIcon],
        ['banner', pendingBanner],
        ['tagBadge', pendingTagBadge],
        ['inviteSplash', pendingInviteSplash],
      ] as const) {
        if (!file) continue;
        const uploaded = await uploadPending(kind, file);
        if (!uploaded.ok) {
          setError(uploaded.message);
          setPending(false);
          return;
        }
        applyCommunityMedia(uploaded.data);
      }

      const result = await updateCommunity({
        path: { community_id: communityId },
        body: {
          name: trimmed,
          description: description.trim(),
          join_mode: joinMode,
          discoverable_on_instance: discoverable,
          tag_name: tagName.trim(),
          tag_color: tagColor.trim(),
          invite_path: invitePath.trim(),
        },
      });
      if (result.error || !result.data) {
        setError(readApiErrorMessage(result.error, 'Could not save settings.'));
        setPending(false);
        return;
      }
      setCommunity(result.data);
      setName(result.data.name);
      setDescription(result.data.description ?? '');
      setTagName(result.data.tag_name ?? '');
      setTagColor(result.data.tag_color?.trim() || '54 210 205');
      setInvitePath(result.data.invite_path ?? '');
      applyCommunityMedia(result.data);
      setPendingIcon(null);
      setPendingBanner(null);
      setPendingTagBadge(null);
      setPendingInviteSplash(null);
      setIconPreview((prev) => {
        if (prev) URL.revokeObjectURL(prev);
        return null;
      });
      setBannerPreview((prev) => {
        if (prev) URL.revokeObjectURL(prev);
        return null;
      });
      setTagBadgePreview((prev) => {
        if (prev) URL.revokeObjectURL(prev);
        return null;
      });
      setInviteSplashPreview((prev) => {
        if (prev) URL.revokeObjectURL(prev);
        return null;
      });
      setMediaBust((v) => v + 1);
    } finally {
      setPending(false);
    }
  };

  const transfer = async () => {
    if (!transferTarget) {
      setError('Pick a member to transfer ownership to.');
      return;
    }
    setPending(true);
    setError(null);
    const result = await transferCommunity({
      path: { community_id: communityId },
      body: { account_id: transferTarget },
    });
    setPending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not transfer ownership.'));
      return;
    }
    setCommunity(result.data);
    setTransferTarget('');
    await refresh();
  };

  const destroy = async () => {
    if (!community) return;
    if (deleteConfirm.trim() !== community.name) {
      setError('Type the community name exactly to confirm deletion.');
      return;
    }
    setPending(true);
    setError(null);
    const result = await deleteCommunity({
      path: { community_id: communityId },
      body: { confirm_name: deleteConfirm.trim() },
    });
    setPending(false);
    if (result.error) {
      setError(readApiErrorMessage(result.error, 'Could not delete community.'));
      return;
    }
    setOpen(false);
    setActiveCommunity('home');
  };

  const transferCandidates = members.filter(
    (m) => m.account_id !== session.account.id && m.role !== 'owner',
  );
  const groups = Array.from(new Set(nav.map((n) => n.group)));
  const iconSrc = iconPreview ?? (iconUrl ? `${iconUrl}?v=${mediaBust}` : null);
  const bannerSrc = bannerPreview ?? (bannerUrl ? `${bannerUrl}?v=${mediaBust}` : null);
  const tagBadgeSrc = tagBadgePreview ?? (tagBadgeUrl ? `${tagBadgeUrl}?v=${mediaBust}` : null);
  const inviteSplashSrc =
    inviteSplashPreview ?? (inviteSplashUrl ? `${inviteSplashUrl}?v=${mediaBust}` : null);
  const initials = name.trim().slice(0, 2).toUpperCase() || 'VX';
  const tagPreview = tagName.trim().slice(0, 8).toUpperCase() || 'TAG';

  return (
    <Portal>
      <div className="fixed inset-0 z-[850] flex bg-app animate-fade-in">
        <div className="flex w-[240px] shrink-0 flex-col border-r border-line/70 bg-panel">
          <div className="flex items-center gap-2 px-4 py-3.5">
            <span className="grid h-6 w-6 place-items-center rounded-md bg-accent/15 font-mono text-2xs font-bold text-accent">
              VX
            </span>
            <div className="min-w-0">
              <div className="truncate font-sans text-[13px] font-semibold text-ink">
                {community?.name ?? 'Community'}
              </div>
              <div className="text-2xs text-ink-3">Settings</div>
            </div>
          </div>
          <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-3">
            {groups.map((g) => (
              <div key={g} className="mb-3">
                <div className="px-2 pb-1">
                  <span className="kicker">{g}</span>
                </div>
                {nav
                  .filter((n) => n.group === g)
                  .map((n) => (
                    <button
                      key={n.id}
                      type="button"
                      onClick={() => setSection(n.id)}
                      className={`group relative flex w-full items-center gap-2.5 rounded-md px-2 py-1.5 text-left text-[13px] font-medium transition-colors ${
                        section === n.id
                          ? 'bg-surface-active text-ink'
                          : 'text-ink-2 hover:bg-surface-hover/70 hover:text-ink'
                      }`}
                    >
                      {section === n.id && <span className="tick" />}
                      <n.Icon
                        size={15}
                        strokeWidth={1.9}
                        className={section === n.id ? 'text-accent' : 'text-ink-3'}
                      />
                      {n.label}
                    </button>
                  ))}
              </div>
            ))}
          </div>
        </div>

        <div className="relative min-w-0 flex-1 overflow-hidden">
          <button
            type="button"
            aria-label="Close"
            onClick={() => setOpen(false)}
            className="absolute right-4 top-4 z-10 grid h-9 w-9 place-items-center rounded-full border border-line/70 bg-panel text-ink-2 hover:bg-surface-hover hover:text-ink"
          >
            <X size={16} />
          </button>

          {section === 'roles' ? (
            <CommunityRolesPanel communityId={communityId} canManage={isOwner} />
          ) : (
            <div className="h-full overflow-y-auto px-8 py-6 pr-16">
              <h1 className="mb-1 text-xl font-semibold text-ink">
                {section === 'overview' && 'Overview'}
                {section === 'members' && 'Members'}
                {section === 'danger' && 'Danger Zone'}
              </h1>
              <p className="mb-6 text-sm text-ink-3">
                {section === 'overview' &&
                  'Server profile, join mode, and discovery on this instance.'}
                {section === 'members' && 'People in this community.'}
                {section === 'danger' && 'Ownership transfer and permanent deletion.'}
              </p>

              {error ? <p className="mb-4 text-sm text-[rgb(var(--danger))]">{error}</p> : null}

              {section === 'overview' && (
                <div className="max-w-xl space-y-8">
                  <section className="space-y-4">
                    <div>
                      <h2 className="text-sm font-semibold text-ink">Server Profile</h2>
                      <p className="mt-0.5 text-[12px] text-ink-3">
                        Name, icon, banner, and description shown across the community.
                      </p>
                    </div>

                    <div
                      className="relative h-28 overflow-hidden rounded-xl border border-line/60 bg-surface"
                      style={
                        bannerSrc
                          ? {
                              backgroundImage: `url(${bannerSrc})`,
                              backgroundSize: 'cover',
                              backgroundPosition: 'center',
                            }
                          : undefined
                      }
                    >
                      {!bannerSrc ? (
                        <div className="absolute inset-0 bg-gradient-to-br from-accent/25 via-surface to-panel" />
                      ) : null}
                      <div className="absolute inset-x-0 bottom-0 flex items-end gap-3 bg-gradient-to-t from-black/55 to-transparent p-3">
                        <div className="grid h-16 w-16 shrink-0 place-items-center overflow-hidden rounded-2xl border-2 border-app/80 bg-panel text-sm font-bold text-ink shadow-md">
                          {iconSrc ? (
                            <img src={iconSrc} alt="" className="h-full w-full object-cover" />
                          ) : (
                            initials
                          )}
                        </div>
                        <div className="min-w-0 pb-1">
                          <p className="truncate text-sm font-semibold text-white drop-shadow">
                            {name.trim() || 'Community'}
                          </p>
                          {description.trim() ? (
                            <p className="line-clamp-2 text-[12px] text-white/80">{description}</p>
                          ) : null}
                        </div>
                      </div>
                    </div>

                    <div className="flex flex-wrap gap-2">
                      <input
                        ref={iconRef}
                        type="file"
                        accept="image/png,image/jpeg,image/gif,image/webp"
                        className="hidden"
                        disabled={!isOwner}
                        onChange={(e) => {
                          pickImage('icon', e.target.files?.[0]);
                          e.target.value = '';
                        }}
                      />
                      <input
                        ref={bannerRef}
                        type="file"
                        accept="image/png,image/jpeg,image/gif,image/webp"
                        className="hidden"
                        disabled={!isOwner}
                        onChange={(e) => {
                          pickImage('banner', e.target.files?.[0]);
                          e.target.value = '';
                        }}
                      />
                      <button
                        type="button"
                        disabled={!isOwner || pending}
                        onClick={() => iconRef.current?.click()}
                        className="inline-flex items-center gap-1.5 rounded-lg border border-line px-3 py-1.5 text-[13px] text-ink-2 hover:bg-surface-hover disabled:opacity-50"
                      >
                        <ImagePlus size={14} />
                        Change icon
                      </button>
                      <button
                        type="button"
                        disabled={!isOwner || pending}
                        onClick={() => bannerRef.current?.click()}
                        className="inline-flex items-center gap-1.5 rounded-lg border border-line px-3 py-1.5 text-[13px] text-ink-2 hover:bg-surface-hover disabled:opacity-50"
                      >
                        <ImagePlus size={14} />
                        Change banner
                      </button>
                    </div>

                    <label className="block text-xs font-medium uppercase tracking-wide text-ink-3">
                      Name
                      <input
                        value={name}
                        onChange={(e) => setName(e.target.value)}
                        disabled={!isOwner}
                        maxLength={100}
                        className="mt-1 w-full rounded-lg border border-line-2/80 bg-surface px-3 py-2 text-sm normal-case tracking-normal text-ink outline-none focus:border-accent/50 disabled:opacity-60"
                      />
                    </label>
                    <label className="block text-xs font-medium uppercase tracking-wide text-ink-3">
                      Description
                      <textarea
                        value={description}
                        onChange={(e) => setDescription(e.target.value)}
                        disabled={!isOwner}
                        rows={3}
                        maxLength={2000}
                        className="mt-1 w-full resize-y rounded-lg border border-line-2/80 bg-surface px-3 py-2 text-sm normal-case tracking-normal text-ink outline-none focus:border-accent/50 disabled:opacity-60"
                      />
                    </label>
                  </section>

                  <section className="space-y-4">
                    <div>
                      <h2 className="text-sm font-semibold text-ink">Server Tag</h2>
                      <p className="mt-0.5 text-[12px] text-ink-3">
                        Short community identity badge — always available, not boost-gated.
                      </p>
                    </div>
                    <div className="flex items-center gap-3 rounded-xl border border-line/60 p-3">
                      <div
                        className="grid h-12 min-w-[3rem] place-items-center rounded-lg px-2 text-xs font-bold text-white shadow-sm"
                        style={{
                          background: tagBadgeSrc
                            ? `center/cover url(${tagBadgeSrc})`
                            : `rgb(${tagColor || '54 210 205'})`,
                        }}
                      >
                        {!tagBadgeSrc ? tagPreview : null}
                      </div>
                      <div className="min-w-0 flex-1 text-[12px] text-ink-3">
                        Preview of how the tag may appear beside the community name.
                      </div>
                    </div>
                    <label className="block text-xs font-medium uppercase tracking-wide text-ink-3">
                      Tag name
                      <input
                        value={tagName}
                        onChange={(e) => setTagName(e.target.value)}
                        disabled={!isOwner}
                        maxLength={8}
                        placeholder="e.g. VOX"
                        className="mt-1 w-full rounded-lg border border-line-2/80 bg-surface px-3 py-2 text-sm normal-case tracking-normal text-ink outline-none focus:border-accent/50 disabled:opacity-60"
                      />
                    </label>
                    <label className="block text-xs font-medium uppercase tracking-wide text-ink-3">
                      Tag color
                      <div className="mt-1 flex items-center gap-3">
                        <input
                          type="color"
                          value={rgbStringToHex(tagColor)}
                          disabled={!isOwner}
                          onChange={(e) => setTagColor(hexToRgbString(e.target.value))}
                          className="h-10 w-10 shrink-0 cursor-pointer rounded-lg border border-line bg-surface p-0.5 disabled:opacity-60"
                        />
                        <span className="text-[12px] text-ink-3">Used when no badge image is set.</span>
                      </div>
                    </label>
                    <div>
                      <input
                        ref={tagBadgeRef}
                        type="file"
                        accept="image/png,image/jpeg,image/gif,image/webp"
                        className="hidden"
                        disabled={!isOwner}
                        onChange={(e) => {
                          pickImage('tagBadge', e.target.files?.[0]);
                          e.target.value = '';
                        }}
                      />
                      <button
                        type="button"
                        disabled={!isOwner || pending}
                        onClick={() => tagBadgeRef.current?.click()}
                        className="inline-flex items-center gap-1.5 rounded-lg border border-line px-3 py-1.5 text-[13px] text-ink-2 hover:bg-surface-hover disabled:opacity-50"
                      >
                        <ImagePlus size={14} />
                        Upload badge
                      </button>
                    </div>
                  </section>

                  <section className="space-y-4">
                    <div>
                      <h2 className="text-sm font-semibold text-ink">Invite presentation</h2>
                      <p className="mt-0.5 text-[12px] text-ink-3">
                        Splash background and a custom invite path on this instance.
                      </p>
                    </div>
                    <div
                      className="relative h-24 overflow-hidden rounded-xl border border-line/60 bg-surface"
                      style={
                        inviteSplashSrc
                          ? {
                              backgroundImage: `url(${inviteSplashSrc})`,
                              backgroundSize: 'cover',
                              backgroundPosition: 'center',
                            }
                          : undefined
                      }
                    >
                      {!inviteSplashSrc ? (
                        <div className="absolute inset-0 bg-gradient-to-br from-panel via-surface to-accent/20" />
                      ) : null}
                      <div className="absolute inset-0 grid place-items-center text-[12px] text-ink-3">
                        Invite splash preview
                      </div>
                    </div>
                    <div>
                      <input
                        ref={inviteSplashRef}
                        type="file"
                        accept="image/png,image/jpeg,image/gif,image/webp"
                        className="hidden"
                        disabled={!isOwner}
                        onChange={(e) => {
                          pickImage('inviteSplash', e.target.files?.[0]);
                          e.target.value = '';
                        }}
                      />
                      <button
                        type="button"
                        disabled={!isOwner || pending}
                        onClick={() => inviteSplashRef.current?.click()}
                        className="inline-flex items-center gap-1.5 rounded-lg border border-line px-3 py-1.5 text-[13px] text-ink-2 hover:bg-surface-hover disabled:opacity-50"
                      >
                        <ImagePlus size={14} />
                        Upload invite splash
                      </button>
                    </div>
                    <label className="block text-xs font-medium uppercase tracking-wide text-ink-3">
                      Custom invite path
                      <input
                        value={invitePath}
                        onChange={(e) => setInvitePath(e.target.value)}
                        disabled={!isOwner}
                        maxLength={48}
                        placeholder="my-community"
                        className="mt-1 w-full rounded-lg border border-line-2/80 bg-surface px-3 py-2 font-mono text-sm normal-case tracking-normal text-ink outline-none focus:border-accent/50 disabled:opacity-60"
                      />
                      <span className="mt-1 block text-[11px] normal-case tracking-normal text-ink-4">
                        Instance-local path for invite links. Leave empty to use the default slug only.
                      </span>
                    </label>
                  </section>

                  <section className="space-y-4">
                    <div>
                      <h2 className="text-sm font-semibold text-ink">Access</h2>
                      <p className="mt-0.5 text-[12px] text-ink-3">
                        How people join and whether this community appears on the instance.
                      </p>
                    </div>
                    <label className="block text-xs font-medium uppercase tracking-wide text-ink-3">
                      Join mode
                      <select
                        value={joinMode}
                        onChange={(e) =>
                          setJoinMode(e.target.value as Exclude<JoinMode, 'application'>)
                        }
                        disabled={!isOwner}
                        className="mt-1 w-full rounded-lg border border-line-2/80 bg-surface px-3 py-2 text-sm normal-case tracking-normal text-ink outline-none focus:border-accent/50 disabled:opacity-60"
                      >
                        <option value="open">Open</option>
                        <option value="invite">Invite only</option>
                      </select>
                    </label>
                    <label className="flex items-center gap-2 text-sm text-ink">
                      <input
                        type="checkbox"
                        checked={discoverable}
                        onChange={(e) => setDiscoverable(e.target.checked)}
                        disabled={!isOwner}
                      />
                      Discoverable on this instance
                    </label>
                  </section>

                  {isOwner ? (
                    <button
                      type="button"
                      disabled={pending}
                      onClick={() => void save()}
                      className="rounded-lg bg-accent px-3 py-2 text-sm font-medium text-app disabled:opacity-60"
                    >
                      {pending ? 'Saving…' : 'Save changes'}
                    </button>
                  ) : (
                    <p className="text-sm text-ink-3">Only the owner can change these settings.</p>
                  )}
                </div>
              )}

              {section === 'members' && (
                <ul className="max-w-lg divide-y divide-line/50 rounded-xl border border-line/60">
                  {members.map((m) => (
                    <li
                      key={m.account_id}
                      className="flex items-center justify-between px-3 py-2.5 text-sm"
                    >
                      <span className="text-ink">
                        {m.nickname.trim() || m.display_name || 'Member'}
                      </span>
                      <span className="text-ink-3">{m.role}</span>
                    </li>
                  ))}
                </ul>
              )}

              {section === 'danger' && isOwner && (
                <div className="max-w-lg space-y-6">
                  <div className="rounded-xl border border-line/60 p-4">
                    <h2 className="mb-2 text-sm font-semibold text-ink">Transfer ownership</h2>
                    <select
                      value={transferTarget}
                      onChange={(e) => setTransferTarget(e.target.value)}
                      className="mb-3 w-full rounded-lg border border-line-2/80 bg-surface px-3 py-2 text-sm text-ink"
                    >
                      <option value="">Select member…</option>
                      {transferCandidates.map((m) => (
                        <option key={m.account_id} value={m.account_id}>
                          {m.nickname.trim() || m.display_name || m.account_id}
                        </option>
                      ))}
                    </select>
                    <button
                      type="button"
                      disabled={pending}
                      onClick={() => void transfer()}
                      className="rounded-lg border border-line px-3 py-2 text-sm text-ink-2 hover:bg-surface-hover"
                    >
                      Transfer
                    </button>
                  </div>
                  <div className="rounded-xl border border-[rgb(var(--danger))]/40 p-4">
                    <h2 className="mb-2 text-sm font-semibold text-[rgb(var(--danger))]">
                      Delete community
                    </h2>
                    <p className="mb-2 text-sm text-ink-3">
                      Type <strong className="text-ink">{community?.name}</strong> to confirm.
                    </p>
                    <input
                      value={deleteConfirm}
                      onChange={(e) => setDeleteConfirm(e.target.value)}
                      className="mb-3 w-full rounded-lg border border-line-2/80 bg-surface px-3 py-2 text-sm text-ink"
                    />
                    <button
                      type="button"
                      disabled={pending}
                      onClick={() => void destroy()}
                      className="rounded-lg bg-[rgb(var(--danger))] px-3 py-2 text-sm font-medium text-white disabled:opacity-60"
                    >
                      Delete forever
                    </button>
                  </div>
                </div>
              )}

              {section === 'danger' && !isOwner && (
                <p className="text-sm text-ink-3">Only the owner can use danger-zone actions.</p>
              )}
            </div>
          )}
        </div>
      </div>
    </Portal>
  );
}
