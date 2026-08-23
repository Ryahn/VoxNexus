import {
  type CommunityMemberResponse,
  type CommunityResponse,
  deleteCommunity,
  getCommunity,
  type JoinMode,
  listCommunityMembers,
  transferCommunity,
  updateCommunity,
} from '@voxnexus/api-client';
import { useEffect, useState } from 'react';
import { useAuth } from '../auth';
import { readApiErrorMessage } from '../lib/apiError';
import { useUI } from '../store';
import { Portal } from './ui/Portal';

type Props = {
  communityId: string;
};

export function CommunitySettingsModal({ communityId }: Props) {
  const { session } = useAuth();
  const open = useUI((s) => s.communitySettingsOpen);
  const setOpen = useUI((s) => s.setCommunitySettingsOpen);
  const setActiveCommunity = useUI((s) => s.setCommunity);
  const [community, setCommunity] = useState<CommunityResponse | null>(null);
  const [members, setMembers] = useState<CommunityMemberResponse[]>([]);
  const [joinMode, setJoinMode] = useState<Exclude<JoinMode, 'application'>>('open');
  const [discoverable, setDiscoverable] = useState(true);
  const [transferTarget, setTransferTarget] = useState('');
  const [deleteConfirm, setDeleteConfirm] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

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
    setCommunity(communityResult.data);
    setJoinMode(communityResult.data.join_mode === 'invite' ? 'invite' : 'open');
    setDiscoverable(communityResult.data.discoverable_on_instance);
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
    void refresh();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setOpen(false);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, communityId, setOpen]);

  if (!open) return null;

  const save = async () => {
    setPending(true);
    setError(null);
    const result = await updateCommunity({
      path: { community_id: communityId },
      body: {
        join_mode: joinMode,
        discoverable_on_instance: discoverable,
      },
    });
    setPending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not save settings.'));
      return;
    }
    setCommunity(result.data);
    setOpen(false);
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

  return (
    <Portal>
      <div className="fixed inset-0 z-[80] grid place-items-center p-4">
        <button
          type="button"
          aria-label="Close"
          className="absolute inset-0 bg-black/55"
          onClick={() => setOpen(false)}
        />
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="community-settings-title"
          className="relative z-10 w-full max-w-md rounded-2xl border border-line/80 bg-panel p-5 shadow-xl"
        >
          <h2 id="community-settings-title" className="mb-1 text-lg font-semibold text-ink">
            Community settings
          </h2>
          <p className="mb-4 text-sm text-ink-3">
            {community ? community.name : 'Loading…'} — who can join, and visibility on this
            instance.
          </p>

          {isOwner ? (
            <>
              <fieldset className="mb-4">
                <legend className="mb-2 text-xs font-medium uppercase tracking-wide text-ink-3">
                  Join mode
                </legend>
                <label className="mb-2 flex cursor-pointer items-start gap-2 rounded-lg border border-line/60 px-3 py-2 hover:bg-surface-hover/40">
                  <input
                    type="radio"
                    name="join-mode"
                    checked={joinMode === 'open'}
                    onChange={() => setJoinMode('open')}
                    className="mt-1"
                  />
                  <span>
                    <span className="block text-[13px] font-medium text-ink">Open</span>
                    <span className="block text-[12px] text-ink-3">
                      Anyone with the community id can join.
                    </span>
                  </span>
                </label>
                <label className="flex cursor-pointer items-start gap-2 rounded-lg border border-line/60 px-3 py-2 hover:bg-surface-hover/40">
                  <input
                    type="radio"
                    name="join-mode"
                    checked={joinMode === 'invite'}
                    onChange={() => setJoinMode('invite')}
                    className="mt-1"
                  />
                  <span>
                    <span className="block text-[13px] font-medium text-ink">Invite only</span>
                    <span className="block text-[12px] text-ink-3">
                      Join requires a valid invite code. Direct join by id is blocked.
                    </span>
                  </span>
                </label>
              </fieldset>

              <label className="mb-4 flex cursor-pointer items-start gap-2 rounded-lg border border-line/60 px-3 py-2 hover:bg-surface-hover/40">
                <input
                  type="checkbox"
                  checked={discoverable}
                  onChange={(e) => setDiscoverable(e.target.checked)}
                  className="mt-1"
                />
                <span>
                  <span className="block text-[13px] font-medium text-ink">Discoverable</span>
                  <span className="block text-[12px] text-ink-3">
                    Show in the instance directory when that feature ships. Independent of join
                    mode.
                  </span>
                </span>
              </label>
            </>
          ) : (
            <p className="mb-4 text-sm text-ink-3">
              Only the community owner can change join settings.
            </p>
          )}

          {isOwner ? (
            <div className="mb-4 rounded-lg border border-dnd/30 bg-dnd/5 p-3">
              <h3 className="mb-2 text-xs font-medium uppercase tracking-wide text-dnd">
                Danger zone
              </h3>
              <label className="mb-3 block text-xs font-medium uppercase tracking-wide text-ink-3">
                Transfer ownership
                <div className="mt-1 flex gap-2">
                  <select
                    value={transferTarget}
                    onChange={(e) => setTransferTarget(e.target.value)}
                    className="min-w-0 flex-1 rounded-lg border border-line-2/80 bg-surface px-2 py-2 text-sm text-ink"
                  >
                    <option value="">Select member…</option>
                    {transferCandidates.map((m) => (
                      <option key={m.account_id} value={m.account_id}>
                        {m.nickname.trim() || m.display_name}
                      </option>
                    ))}
                  </select>
                  <button
                    type="button"
                    disabled={pending || !transferTarget}
                    onClick={() => void transfer()}
                    className="rounded-lg border border-line px-3 py-2 text-sm text-ink-2 hover:bg-surface-hover disabled:opacity-50"
                  >
                    Transfer
                  </button>
                </div>
              </label>
              <label className="block text-xs font-medium uppercase tracking-wide text-ink-3">
                Delete community
                <input
                  value={deleteConfirm}
                  onChange={(e) => setDeleteConfirm(e.target.value)}
                  placeholder={community?.name ?? 'Community name'}
                  className="mt-1 w-full rounded-lg border border-dnd/40 bg-surface px-3 py-2 text-sm text-ink outline-none focus:border-dnd/60"
                />
                <button
                  type="button"
                  disabled={pending || !community}
                  onClick={() => void destroy()}
                  className="mt-2 rounded-lg bg-dnd px-3 py-2 text-sm font-medium text-app disabled:opacity-50"
                >
                  Delete forever
                </button>
              </label>
            </div>
          ) : null}

          {error ? <p className="mb-3 text-sm text-[rgb(var(--danger))]">{error}</p> : null}

          <div className="flex justify-end gap-2">
            <button
              type="button"
              onClick={() => setOpen(false)}
              className="rounded-lg px-3 py-2 text-sm text-ink-2 hover:bg-surface-hover"
            >
              {isOwner ? 'Cancel' : 'Close'}
            </button>
            {isOwner ? (
              <button
                type="button"
                disabled={pending || !community}
                onClick={() => void save()}
                className="rounded-lg bg-accent px-3 py-2 text-sm font-medium text-app disabled:opacity-60"
              >
                {pending ? 'Saving…' : 'Save'}
              </button>
            ) : null}
          </div>
        </div>
      </div>
    </Portal>
  );
}
