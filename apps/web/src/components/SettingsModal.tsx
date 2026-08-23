import {
  changeMyEmail,
  changeMyPassword,
  getInstanceSettings,
  getMyProfile,
  updateInstanceSettings,
  updateMyProfile,
  uploadMyAvatar,
  uploadMyBanner,
} from '@voxnexus/api-client';
import {
  Bell,
  Check,
  IdCard,
  Keyboard,
  Languages,
  LogOut,
  type LucideIcon,
  Mic2,
  Palette,
  Server,
  SlidersHorizontal,
  UserCircle,
  X,
} from 'lucide-react';
import { type ReactNode, useEffect, useState } from 'react';
import { useAuth } from '../auth';
import { roles } from '../data/roles';
import { me } from '../data/users';
import { readApiErrorMessage } from '../lib/apiError';
import { bannerGradient } from '../lib/avatar';
import { type PresenceState, presenceLabel, usePresence } from '../presence';
import { useUI } from '../store';
import { Avatar } from './ui/Avatar';
import { Portal } from './ui/Portal';

const credentials = { credentials: 'include' as const };

type Section =
  | 'account'
  | 'profile'
  | 'instance'
  | 'appearance'
  | 'voice'
  | 'notifications'
  | 'keyboard'
  | 'language'
  | 'advanced';

const nav: { id: Section; label: string; Icon: LucideIcon; group: string }[] = [
  { id: 'account', label: 'My Account', Icon: UserCircle, group: 'USER SETTINGS' },
  { id: 'profile', label: 'Profile', Icon: IdCard, group: 'USER SETTINGS' },
  { id: 'appearance', label: 'Appearance', Icon: Palette, group: 'APP SETTINGS' },
  { id: 'voice', label: 'Voice & Video', Icon: Mic2, group: 'APP SETTINGS' },
  { id: 'notifications', label: 'Notifications', Icon: Bell, group: 'APP SETTINGS' },
  { id: 'keyboard', label: 'Keyboard Shortcuts', Icon: Keyboard, group: 'APP SETTINGS' },
  { id: 'language', label: 'Language', Icon: Languages, group: 'APP SETTINGS' },
  { id: 'advanced', label: 'Advanced', Icon: SlidersHorizontal, group: 'APP SETTINGS' },
];

const ACCENTS: { name: string; rgb: string }[] = [
  { name: 'Teal', rgb: '54 210 205' },
  { name: 'Electric', rgb: '76 159 254' },
  { name: 'Violet', rgb: '138 124 246' },
  { name: 'Magenta', rgb: '240 97 168' },
  { name: 'Emerald', rgb: '63 202 122' },
  { name: 'Amber', rgb: '240 180 41' },
];

export function SettingsModal() {
  const open = useUI((s) => s.settingsOpen);
  const setOpen = useUI((s) => s.setSettingsOpen);
  const { signOut, session } = useAuth();
  const [section, setSection] = useState<Section>('account');

  const adminNav = session.account.is_instance_admin
    ? [{ id: 'instance' as Section, label: 'Instance', Icon: Server, group: 'INSTANCE' }]
    : [];
  const allNav = [...nav, ...adminNav];

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && setOpen(false);
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, [open, setOpen]);

  if (!open) return null;

  const groups = Array.from(new Set(allNav.map((n) => n.group)));

  return (
    <Portal>
      <div className="fixed inset-0 z-[850] flex bg-app animate-fade-in">
        {/* left nav */}
        <div className="flex w-[240px] shrink-0 flex-col border-r border-line/70 bg-panel">
          <div className="flex items-center gap-2 px-4 py-3.5">
            <span className="grid h-6 w-6 place-items-center rounded-md bg-accent/15 font-mono text-2xs font-bold text-accent">
              VX
            </span>
            <span className="font-sans text-[13px] font-semibold text-ink">Settings</span>
          </div>
          <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-3">
            {groups.map((g) => (
              <div key={g} className="mb-3">
                <div className="px-2 pb-1">
                  <span className="kicker">{g}</span>
                </div>
                {allNav
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
            <div className="my-2 h-px bg-line-2/60" />
            <button
              type="button"
              onClick={() => {
                void signOut();
              }}
              className="flex w-full items-center gap-2.5 rounded-md px-2 py-1.5 text-left text-[13px] font-medium text-dnd transition-colors hover:bg-dnd/12"
            >
              <LogOut size={15} strokeWidth={1.9} /> Log Out
            </button>
          </div>
        </div>

        {/* content */}
        <div className="relative min-w-0 flex-1 overflow-y-auto">
          <button
            type="button"
            onClick={() => setOpen(false)}
            aria-label="Close settings"
            className="absolute right-5 top-5 z-10 flex flex-col items-center gap-1 text-ink-3 transition-colors hover:text-ink"
          >
            <span className="grid h-9 w-9 place-items-center rounded-full border border-line-2/70 hover:border-line-2">
              <X size={18} />
            </span>
            <kbd className="font-mono text-3xs">ESC</kbd>
          </button>

          <div className="mx-auto max-w-2xl px-6 py-8 md:px-10">
            {section === 'account' && <AccountPanel />}
            {section === 'profile' && <ProfilePanel />}
            {section === 'instance' && <InstancePanel />}
            {section === 'appearance' && <AppearancePanel />}
            {section === 'voice' && <VoicePanelSettings />}
            {section === 'notifications' && <NotificationsPanel />}
            {section === 'keyboard' && <KeyboardPanel />}
            {section === 'language' && <LanguagePanel />}
            {section === 'advanced' && <AdvancedPanel />}
          </div>
        </div>
      </div>
    </Portal>
  );
}

/* ---------- shared bits ---------- */

function H({ children }: { children: ReactNode }) {
  return <h1 className="mb-5 font-sans text-[19px] font-bold text-ink">{children}</h1>;
}

function SectionLabel({ children }: { children: React.ReactNode }) {
  return (
    <div className="mb-2 mt-6 font-mono text-3xs uppercase tracking-[0.14em] text-ink-3">
      {children}
    </div>
  );
}

function Toggle({ on, onClick }: { on: boolean; onClick: () => void }) {
  return (
    <button
      role="switch"
      aria-checked={on}
      onClick={onClick}
      className={`relative h-[22px] w-10 shrink-0 rounded-full transition-colors duration-150 ${
        on ? 'bg-accent' : 'bg-surface-active'
      }`}
    >
      <span
        className={`absolute top-0.5 h-[18px] w-[18px] rounded-full bg-app transition-transform duration-150 ${
          on ? 'translate-x-[20px]' : 'translate-x-0.5'
        }`}
      />
    </button>
  );
}

function ToggleRow({
  label,
  desc,
  on,
  onClick,
}: {
  label: string;
  desc?: string;
  on: boolean;
  onClick: () => void;
}) {
  return (
    <div className="flex items-center gap-4 border-b border-line/60 py-3 last:border-0">
      <div className="min-w-0 flex-1">
        <div className="text-[13.5px] font-medium text-ink">{label}</div>
        {desc && <div className="mt-0.5 text-[12.5px] leading-snug text-ink-3">{desc}</div>}
      </div>
      <Toggle on={on} onClick={onClick} />
    </div>
  );
}

function FieldRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between rounded-lg border border-line-2/50 bg-input px-3 py-2.5">
      <div>
        <div className="kicker">{label}</div>
        <div className="mt-0.5 text-[13.5px] text-ink">{value}</div>
      </div>
      <button className="rounded-md border border-line-2/60 px-2.5 py-1 text-[12px] font-medium text-ink-2 hover:bg-surface-hover hover:text-ink">
        Edit
      </button>
    </div>
  );
}

function Select({ label, options }: { label: string; options: string[] }) {
  return (
    <label className="block">
      <span className="kicker">{label}</span>
      <select className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] text-ink outline-none focus:border-accent/60">
        {options.map((o) => (
          <option key={o}>{o}</option>
        ))}
      </select>
    </label>
  );
}

/* ---------- panels ---------- */

function InstancePanel() {
  const [name, setName] = useState('');
  const [publicUrl, setPublicUrl] = useState('');
  const [registrationMode, setRegistrationMode] = useState<'open' | 'invite' | 'closed'>('open');
  const [communityMode, setCommunityMode] = useState<'open' | 'admin_only' | 'single'>('open');
  const [modeLocked, setModeLocked] = useState(false);
  const [oidcEnabled, setOidcEnabled] = useState(false);
  const [oidcIssuer, setOidcIssuer] = useState('');
  const [oidcClientId, setOidcClientId] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    getInstanceSettings(credentials)
      .then((result) => {
        if (cancelled || !result.data) {
          return;
        }
        const data = result.data;
        setName(data.name);
        setPublicUrl(data.public_url);
        setRegistrationMode(data.registration_mode);
        setCommunityMode(data.community_creation_mode);
        setModeLocked(data.community_creation_mode_locked);
        setOidcEnabled(data.oidc_enabled);
        setOidcIssuer(data.oidc_issuer ?? '');
        setOidcClientId(data.oidc_client_id ?? '');
      })
      .catch(() => {
        if (!cancelled) {
          setError('Failed to load instance settings.');
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  async function onSave() {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const result = await updateInstanceSettings({
        body: {
          name,
          public_url: publicUrl,
          registration_mode: registrationMode,
          community_creation_mode: modeLocked ? undefined : communityMode,
          oidc_enabled: oidcEnabled,
          oidc_issuer: oidcIssuer.trim() ? oidcIssuer.trim() : null,
          oidc_client_id: oidcClientId.trim() ? oidcClientId.trim() : null,
        },
        ...credentials,
      });
      if (result.data) {
        setNotice('Instance settings saved.');
        setModeLocked(result.data.community_creation_mode_locked);
        return;
      }
      setError(readApiErrorMessage(result.error, 'Save failed.'));
    } catch {
      setError('Save failed.');
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <H>Instance</H>
      <p className="mb-4 text-[13px] text-ink-3">
        Registration and community creation policy for this server.
      </p>
      <div className="space-y-3 rounded-xl border border-line-2/60 bg-panel-2 p-4">
        <label className="block">
          <span className="kicker">Instance name</span>
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] text-ink outline-none focus:border-accent/60"
          />
        </label>
        <label className="block">
          <span className="kicker">Public URL</span>
          <input
            value={publicUrl}
            onChange={(e) => setPublicUrl(e.target.value)}
            className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] text-ink outline-none focus:border-accent/60"
          />
        </label>
        <label className="block">
          <span className="kicker">Registration</span>
          <select
            value={registrationMode}
            onChange={(e) => setRegistrationMode(e.target.value as 'open' | 'invite' | 'closed')}
            className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] text-ink outline-none focus:border-accent/60"
          >
            <option value="open">Open — anyone can register</option>
            <option value="invite">Invite only (registration blocked until invites ship)</option>
            <option value="closed">Closed — no new registrations</option>
          </select>
        </label>
        <label className="block">
          <span className="kicker">Community creation</span>
          <select
            value={communityMode}
            disabled={modeLocked}
            onChange={(e) => setCommunityMode(e.target.value as 'open' | 'admin_only' | 'single')}
            className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] text-ink outline-none focus:border-accent/60 disabled:opacity-60"
          >
            <option value="open">Open — any member can create</option>
            <option value="admin_only">Admin only</option>
            <option value="single">Single — one community per instance (personal install)</option>
          </select>
          {modeLocked ? (
            <p className="mt-1 text-[12px] text-ink-3">
              Locked by operator env (`COMMUNITY_CREATION_MODE_LOCKED`).
            </p>
          ) : null}
        </label>
        <SectionLabel>OIDC / SSO</SectionLabel>
        <ToggleRow
          label="Enable SSO"
          desc="If OIDC_ISSUER is set in config, issuer/client ID sync here on restart. Or set them only here and leave config empty. Secret is always OIDC_CLIENT_SECRET."
          on={oidcEnabled}
          onClick={() => setOidcEnabled(!oidcEnabled)}
        />
        <label className="block">
          <span className="kicker">Issuer URL</span>
          <input
            value={oidcIssuer}
            onChange={(e) => setOidcIssuer(e.target.value)}
            placeholder="https://idp.example.com"
            className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] text-ink outline-none focus:border-accent/60"
          />
        </label>
        <label className="block">
          <span className="kicker">Client ID</span>
          <input
            value={oidcClientId}
            onChange={(e) => setOidcClientId(e.target.value)}
            className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] text-ink outline-none focus:border-accent/60"
          />
        </label>
        {error ? <p className="text-[13px] text-dnd">{error}</p> : null}
        {notice ? <p className="text-[13px] text-online">{notice}</p> : null}
        <button
          type="button"
          disabled={busy}
          onClick={() => void onSave()}
          className="rounded-lg bg-accent px-3 py-2 text-[13px] font-semibold text-app hover:brightness-110 disabled:opacity-60"
        >
          {busy ? 'Saving…' : 'Save instance settings'}
        </button>
      </div>
    </>
  );
}

function AccountPanel() {
  const { session, refresh } = useAuth();
  const email = session.account.email ?? '';
  const userRoles = me.roleIds.map((id) => roles[id]).filter(Boolean);

  const [newEmail, setNewEmail] = useState(email);
  const [emailCurrentPassword, setEmailCurrentPassword] = useState('');
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [revokeOthers, setRevokeOthers] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    setNewEmail(email);
  }, [email]);

  async function onChangeEmail() {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const result = await changeMyEmail({
        body: { email: newEmail, current_password: emailCurrentPassword },
        ...credentials,
      });
      if (result.data) {
        await refresh();
        setEmailCurrentPassword('');
        setNotice('Email updated.');
        return;
      }
      setError(readApiErrorMessage(result.error, 'Email change failed.'));
    } catch {
      setError('Email change failed.');
    } finally {
      setBusy(false);
    }
  }

  async function onChangePassword() {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const result = await changeMyPassword({
        body: {
          current_password: currentPassword,
          new_password: newPassword,
          revoke_other_sessions: revokeOthers,
        },
        ...credentials,
      });
      if (!result.error) {
        setCurrentPassword('');
        setNewPassword('');
        setRevokeOthers(false);
        setNotice('Password updated.');
        return;
      }
      setError(readApiErrorMessage(result.error, 'Password change failed.'));
    } catch {
      setError('Password change failed.');
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <H>My Account</H>
      <div className="overflow-hidden rounded-xl border border-line-2/60 bg-panel-2">
        <div className="h-20" style={{ background: bannerGradient(me.bannerSeed) }}>
          <div className="h-full w-full grid-veil opacity-30" />
        </div>
        <div className="px-4 pb-4">
          <div className="-mt-8 mb-3 flex items-end gap-3">
            <div className="rounded-[34%] border-[3px] border-panel-2">
              <Avatar
                user={me}
                size={64}
                rounded="rounded-[30%]"
                showPresence
                ring="rgb(var(--bg-panel-2))"
              />
            </div>
            <div className="mb-1">
              <div className="font-sans text-[17px] font-bold text-ink">{me.displayName}</div>
              <div className="font-mono text-[12px] text-ink-3">@{me.username}</div>
            </div>
          </div>
          <div className="grid gap-2 sm:grid-cols-2">
            <FieldRow label="Display Name" value={me.displayName} />
            <FieldRow label="Username" value={`@${me.username}`} />
            <FieldRow label="Email" value={email || '—'} />
            <FieldRow label="Pronouns" value={me.pronouns ?? '—'} />
          </div>
          <div className="mt-2 flex flex-wrap gap-1.5">
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
                />{' '}
                {r.name}
              </span>
            ))}
          </div>
        </div>
      </div>

      <SectionLabel>Change email</SectionLabel>
      <form
        className="space-y-3 rounded-xl border border-line-2/50 bg-panel-2 p-4"
        onSubmit={(event) => {
          event.preventDefault();
          void onChangeEmail();
        }}
      >
        <label className="block">
          <span className="kicker">New email</span>
          <input
            type="email"
            name="email"
            autoComplete="email"
            value={newEmail}
            disabled={busy}
            onChange={(event) => setNewEmail(event.target.value)}
            className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] text-ink outline-none focus:border-accent/60"
          />
        </label>
        <label className="block">
          <span className="kicker">Current password</span>
          <input
            type="password"
            name="current_password"
            autoComplete="current-password"
            value={emailCurrentPassword}
            disabled={busy}
            onChange={(event) => setEmailCurrentPassword(event.target.value)}
            className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] text-ink outline-none focus:border-accent/60"
          />
        </label>
        <button
          type="submit"
          disabled={busy || !newEmail || !emailCurrentPassword}
          className="rounded-lg bg-accent px-3 py-2 text-[13px] font-semibold text-app hover:brightness-110 disabled:opacity-60"
        >
          Update email
        </button>
      </form>

      <SectionLabel>Change password</SectionLabel>
      <form
        className="space-y-3 rounded-xl border border-line-2/50 bg-panel-2 p-4"
        onSubmit={(event) => {
          event.preventDefault();
          void onChangePassword();
        }}
      >
        <label className="block">
          <span className="kicker">Current password</span>
          <input
            type="password"
            name="current_password"
            autoComplete="current-password"
            value={currentPassword}
            disabled={busy}
            onChange={(event) => setCurrentPassword(event.target.value)}
            className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] text-ink outline-none focus:border-accent/60"
          />
        </label>
        <label className="block">
          <span className="kicker">New password</span>
          <input
            type="password"
            name="new_password"
            autoComplete="new-password"
            minLength={8}
            value={newPassword}
            disabled={busy}
            onChange={(event) => setNewPassword(event.target.value)}
            className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] text-ink outline-none focus:border-accent/60"
          />
        </label>
        <label className="flex items-center gap-2 text-[13px] text-ink-2">
          <input
            type="checkbox"
            checked={revokeOthers}
            disabled={busy}
            onChange={(event) => setRevokeOthers(event.target.checked)}
            className="rounded border-line-2"
          />
          Sign out other sessions
        </label>
        <button
          type="submit"
          disabled={busy || !currentPassword || newPassword.length < 8}
          className="rounded-lg bg-accent px-3 py-2 text-[13px] font-semibold text-app hover:brightness-110 disabled:opacity-60"
        >
          Update password
        </button>
      </form>

      {error ? <p className="mt-3 text-[13px] text-dnd">{error}</p> : null}
      {notice ? <p className="mt-3 text-[13px] text-online">{notice}</p> : null}
    </>
  );
}

function ProfilePanel() {
  const { setStatus, online } = usePresence();
  const [displayName, setDisplayName] = useState('');
  const [bio, setBio] = useState('');
  const [customStatus, setCustomStatus] = useState('');
  const [presenceStatus, setPresenceStatus] = useState<PresenceState>('online');
  const [avatarUrl, setAvatarUrl] = useState<string | null>(null);
  const [bannerUrl, setBannerUrl] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [bump, setBump] = useState(0);

  useEffect(() => {
    let cancelled = false;
    getMyProfile(credentials)
      .then((result) => {
        if (cancelled || !result.data) {
          return;
        }
        setDisplayName(result.data.display_name);
        setBio(result.data.bio);
        setCustomStatus(result.data.custom_status);
        setPresenceStatus(result.data.presence_status as PresenceState);
        setAvatarUrl(result.data.avatar_url ?? null);
        setBannerUrl(result.data.banner_url ?? null);
      })
      .catch(() => {
        if (!cancelled) {
          setError('Could not load profile.');
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  async function onSave() {
    setBusy(true);
    setError(null);
    try {
      const result = await updateMyProfile({
        body: {
          display_name: displayName,
          bio,
          custom_status: customStatus,
        },
        ...credentials,
      });
      if (!result.data) {
        setError('Save failed.');
      }
    } catch {
      setError('Save failed.');
    } finally {
      setBusy(false);
    }
  }

  async function onUpload(kind: 'avatar' | 'banner', file: File | undefined) {
    if (!file) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const result =
        kind === 'avatar'
          ? await uploadMyAvatar({
              body: file,
              headers: { 'Content-Type': 'application/octet-stream' },
              ...credentials,
            })
          : await uploadMyBanner({
              body: file,
              headers: { 'Content-Type': 'application/octet-stream' },
              ...credentials,
            });
      if (result.data) {
        setDisplayName(result.data.display_name);
        setBio(result.data.bio);
        setAvatarUrl(result.data.avatar_url ?? null);
        setBannerUrl(result.data.banner_url ?? null);
        setBump((value) => value + 1);
      } else {
        setError('Upload failed (check size and image type).');
      }
    } catch {
      setError('Upload failed.');
    } finally {
      setBusy(false);
    }
  }

  const avatarSrc = avatarUrl ? `${avatarUrl}?v=${bump}` : null;
  const bannerSrc = bannerUrl ? `${bannerUrl}?v=${bump}` : null;

  return (
    <>
      <H>Profile</H>
      <div className="mb-4 overflow-hidden rounded-xl border border-line-2/60 bg-panel-2">
        <div
          className="h-24 bg-surface"
          style={
            bannerSrc
              ? { backgroundImage: `url(${bannerSrc})`, backgroundSize: 'cover' }
              : undefined
          }
        />
        <div className="flex items-end gap-3 px-4 pb-4">
          <div className="-mt-8 h-16 w-16 overflow-hidden rounded-[30%] border-[3px] border-panel-2 bg-surface">
            {avatarSrc ? (
              <img src={avatarSrc} alt="" className="h-full w-full object-cover" />
            ) : null}
          </div>
          <div className="mb-1 min-w-0">
            <div className="truncate font-sans text-[17px] font-bold text-ink">
              {displayName || 'Display name'}
            </div>
            <div className="truncate text-[12.5px] text-ink-3">{bio || 'No bio yet.'}</div>
          </div>
        </div>
      </div>
      <div className="space-y-3">
        <label className="block">
          <span className="kicker">Display name</span>
          <input
            value={displayName}
            maxLength={64}
            onChange={(event) => setDisplayName(event.target.value)}
            className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] text-ink outline-none focus:border-accent/60"
          />
        </label>
        <label className="block">
          <span className="kicker">About Me</span>
          <textarea
            value={bio}
            maxLength={500}
            rows={3}
            onChange={(event) => setBio(event.target.value)}
            className="mt-1 w-full resize-none rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] text-ink outline-none focus:border-accent/60"
          />
        </label>
        <div>
          <span className="kicker">Status</span>
          <div className="mt-2 flex flex-wrap gap-2">
            {(['online', 'idle', 'dnd', 'invisible'] as PresenceState[]).map((status) => (
              <button
                key={status}
                type="button"
                disabled={busy}
                onClick={() => {
                  setPresenceStatus(status);
                  void setStatus(status, customStatus);
                }}
                className={`rounded-md border px-2.5 py-1.5 text-[12.5px] font-medium transition-colors ${
                  presenceStatus === status
                    ? 'border-accent/50 bg-accent/15 text-accent'
                    : 'border-line-2/60 text-ink-2 hover:text-ink'
                }`}
              >
                {presenceLabel(status)}
              </button>
            ))}
          </div>
        </div>
        <label className="block">
          <span className="kicker">Custom status</span>
          <input
            value={customStatus}
            maxLength={128}
            onChange={(event) => setCustomStatus(event.target.value)}
            placeholder="What are you up to?"
            className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] text-ink outline-none focus:border-accent/60"
          />
        </label>
        {online.length > 0 ? (
          <div className="rounded-lg border border-line-2/50 bg-app/60 p-3">
            <div className="kicker mb-2">Online on this instance</div>
            <ul className="space-y-1 text-[12.5px] text-ink-2">
              {online.map((entry) => (
                <li key={entry.accountId} className="truncate font-mono text-3xs">
                  {entry.accountId.slice(0, 8)}… — {presenceLabel(entry.status, entry.customStatus)}
                </li>
              ))}
            </ul>
          </div>
        ) : null}
        <div className="grid gap-2 sm:grid-cols-2">
          <label className="block">
            <span className="kicker">Avatar</span>
            <input
              type="file"
              accept="image/png,image/jpeg,image/gif,image/webp"
              disabled={busy}
              className="mt-1 block w-full text-[12.5px] text-ink-2 file:mr-3 file:rounded-md file:border-0 file:bg-surface file:px-2.5 file:py-1.5 file:text-[12px] file:font-medium file:text-ink"
              onChange={(event) => void onUpload('avatar', event.target.files?.[0])}
            />
          </label>
          <label className="block">
            <span className="kicker">Banner</span>
            <input
              type="file"
              accept="image/png,image/jpeg,image/gif,image/webp"
              disabled={busy}
              className="mt-1 block w-full text-[12.5px] text-ink-2 file:mr-3 file:rounded-md file:border-0 file:bg-surface file:px-2.5 file:py-1.5 file:text-[12px] file:font-medium file:text-ink"
              onChange={(event) => void onUpload('banner', event.target.files?.[0])}
            />
          </label>
        </div>
        {error ? <p className="text-[13px] text-dnd">{error}</p> : null}
        <button
          type="button"
          disabled={busy}
          onClick={() => void onSave()}
          className="rounded-lg bg-accent px-3 py-2 text-[13px] font-semibold text-app hover:brightness-110 disabled:opacity-60"
        >
          {busy ? 'Saving…' : 'Save profile'}
        </button>
      </div>
    </>
  );
}

function AppearancePanel() {
  const compact = useUI((s) => s.compact);
  const toggleCompact = useUI((s) => s.toggleCompact);
  const [accent, setAccent] = useState<string>(() =>
    typeof window === 'undefined'
      ? ACCENTS[0].rgb
      : getComputedStyle(document.documentElement).getPropertyValue('--accent').trim() ||
        ACCENTS[0].rgb,
  );
  const [reduceMotion, setReduceMotion] = useState(false);
  const [glow, setGlow] = useState(true);

  const applyAccent = (rgb: string) => {
    document.documentElement.style.setProperty('--accent', rgb);
    setAccent(rgb);
  };

  return (
    <>
      <H>Appearance</H>

      <SectionLabel>Accent Color</SectionLabel>
      <p className="mb-3 text-[12.5px] text-ink-3">
        The single restrained highlight used across selection, focus, and status.
      </p>
      <div className="flex flex-wrap gap-2.5">
        {ACCENTS.map((a) => {
          const active = accent === a.rgb;
          return (
            <button
              key={a.name}
              onClick={() => applyAccent(a.rgb)}
              aria-label={a.name}
              className="group flex flex-col items-center gap-1.5"
            >
              <span
                className="grid h-10 w-10 place-items-center rounded-xl transition-transform duration-150 group-hover:scale-105"
                style={{
                  background: `linear-gradient(150deg, rgb(${a.rgb}), rgb(${a.rgb} / 0.55))`,
                  boxShadow: active
                    ? `0 0 0 2px rgb(var(--bg-app)), 0 0 0 3.5px rgb(${a.rgb})`
                    : 'none',
                }}
              >
                {active && <Check size={16} className="text-app" strokeWidth={3} />}
              </span>
              <span className={`font-mono text-3xs ${active ? 'text-ink' : 'text-ink-3'}`}>
                {a.name}
              </span>
            </button>
          );
        })}
      </div>

      <SectionLabel>Message Display</SectionLabel>
      <div className="rounded-xl border border-line-2/50 bg-panel-2 px-4">
        <ToggleRow
          label="Compact Mode"
          desc="Denser message layout with inline timestamps."
          on={compact}
          onClick={toggleCompact}
        />
        <ToggleRow
          label="Subtle Edge Glow"
          desc="Faint accent glow on focused inputs and active surfaces."
          on={glow}
          onClick={() => setGlow((v) => !v)}
        />
        <ToggleRow
          label="Reduce Motion"
          desc="Minimize non-essential animations and transitions."
          on={reduceMotion}
          onClick={() => setReduceMotion((v) => !v)}
        />
      </div>

      <SectionLabel>Zoom</SectionLabel>
      <Select label="Interface Scale" options={['90%', '100%', '110%', '125%']} />
    </>
  );
}

function VoicePanelSettings() {
  const muted = useUI((s) => s.muted);
  const toggleMute = useUI((s) => s.toggleMute);
  return (
    <>
      <H>Voice & Video</H>
      <div className="grid gap-3 sm:grid-cols-2">
        <Select label="Input Device" options={['Default — Shure MV7', 'System Microphone']} />
        <Select label="Output Device" options={['Default — Studio Monitors', 'Headphones']} />
      </div>
      <SectionLabel>Mic Test</SectionLabel>
      <div className="flex items-center gap-1 rounded-lg border border-line-2/50 bg-input p-3">
        {Array.from({ length: 28 }).map((_, i) => (
          <span
            key={i}
            className="h-4 flex-1 rounded-sm"
            style={{
              background: i < 11 ? 'rgb(var(--accent))' : 'rgb(var(--surface-active))',
              opacity: i < 11 ? 1 - i * 0.05 : 1,
            }}
          />
        ))}
      </div>
      <SectionLabel>Options</SectionLabel>
      <div className="rounded-xl border border-line-2/50 bg-panel-2 px-4">
        <ToggleRow
          label="Mute on Join"
          desc="Start every voice session muted."
          on={muted}
          onClick={toggleMute}
        />
        <ToggleRow label="Echo Cancellation" on onClick={() => {}} />
        <ToggleRow
          label="Noise Suppression"
          desc="Filter background noise from your input."
          on
          onClick={() => {}}
        />
      </div>
    </>
  );
}

function NotificationsPanel() {
  const rows = [
    {
      label: 'Enable Desktop Notifications',
      desc: 'Show a system notification for new activity.',
      on: true,
    },
    { label: 'Mentions Only', desc: 'Only notify me when I’m directly @mentioned.', on: false },
    { label: 'Sounds', desc: 'Play a sound for messages, calls, and joins.', on: true },
    {
      label: 'Community Announcements',
      desc: 'Notify me about announcement channels I follow.',
      on: true,
    },
    { label: 'Friend Requests', desc: 'Notify me when someone adds me.', on: true },
  ];
  return (
    <>
      <H>Notifications</H>
      <div className="rounded-xl border border-line-2/50 bg-panel-2 px-4">
        {rows.map((r) => (
          <StatefulToggleRow key={r.label} {...r} />
        ))}
      </div>
    </>
  );
}

function StatefulToggleRow({ label, desc, on }: { label: string; desc?: string; on: boolean }) {
  const [v, setV] = useState(on);
  return <ToggleRow label={label} desc={desc} on={v} onClick={() => setV((x) => !x)} />;
}

function KeyboardPanel() {
  const shortcuts: [string, string][] = [
    ['Quick Switcher / Search', 'Ctrl K'],
    ['Open Inbox', 'Ctrl B'],
    ['Toggle Member List', 'Ctrl U'],
    ['Mark Channel Read', 'Esc'],
    ['Mark Server Read', 'Shift Esc'],
    ['Next / Previous Channel', 'Alt ↑ / ↓'],
    ['Toggle Mute', 'Ctrl Shift M'],
    ['Toggle Deafen', 'Ctrl Shift D'],
    ['Upload File', 'Ctrl Shift U'],
    ['Open Settings', 'Ctrl ,'],
  ];
  return (
    <>
      <H>Keyboard Shortcuts</H>
      <div className="rounded-xl border border-line-2/50 bg-panel-2 px-4">
        {shortcuts.map(([label, keys]) => (
          <div
            key={label}
            className="flex items-center justify-between border-b border-line/60 py-2.5 last:border-0"
          >
            <span className="text-[13.5px] text-ink-2">{label}</span>
            <span className="flex gap-1">
              {keys.split(' ').map((k, i) => (
                <kbd
                  key={i}
                  className="rounded border border-line-2/60 bg-input px-1.5 py-0.5 font-mono text-3xs text-ink"
                >
                  {k}
                </kbd>
              ))}
            </span>
          </div>
        ))}
      </div>
    </>
  );
}

function LanguagePanel() {
  const langs = [
    'English (US)',
    'Español',
    '日本語',
    'Deutsch',
    'Français',
    '中文 (简体)',
    'Português',
    'Русский',
  ];
  const [sel, setSel] = useState(langs[0]);
  return (
    <>
      <H>Language</H>
      <div className="grid gap-2 sm:grid-cols-2">
        {langs.map((l) => (
          <button
            key={l}
            onClick={() => setSel(l)}
            className={`flex items-center justify-between rounded-lg border px-3 py-2.5 text-left text-[13.5px] transition-colors ${
              sel === l
                ? 'border-accent/50 bg-accent/10 text-ink'
                : 'border-line-2/50 bg-panel-2 text-ink-2 hover:bg-surface-hover'
            }`}
          >
            {l}
            {sel === l && <Check size={15} className="text-accent" />}
          </button>
        ))}
      </div>
    </>
  );
}

function AdvancedPanel() {
  return (
    <>
      <H>Advanced</H>
      <div className="rounded-xl border border-line-2/50 bg-panel-2 px-4">
        <StatefulToggleRow
          label="Developer Mode"
          desc="Enable right-click IDs and the API inspector."
          on={false}
        />
        <StatefulToggleRow
          label="Hardware Acceleration"
          desc="Use your GPU to render the client."
          on
        />
        <StatefulToggleRow
          label="Send Telemetry"
          desc="Share anonymous usage data to improve VOX."
          on={false}
        />
      </div>
      <SectionLabel>Data</SectionLabel>
      <div className="flex flex-wrap gap-2">
        <button className="rounded-lg border border-line-2/60 px-3 py-2 text-[13px] font-medium text-ink-2 hover:bg-surface-hover hover:text-ink">
          Export My Data
        </button>
        <button className="rounded-lg border border-dnd/40 px-3 py-2 text-[13px] font-medium text-dnd hover:bg-dnd/12">
          Clear Local Cache
        </button>
      </div>
      <p className="mt-6 font-mono text-3xs text-ink-4">
        VOX 0.7.0-rc.1 · build 20260822 · self-hosted
      </p>
    </>
  );
}
