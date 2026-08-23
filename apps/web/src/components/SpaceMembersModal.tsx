import {
  addSpaceMember,
  type CommunityMemberResponse,
  getCommunity,
  listCommunityMembers,
  listSpaceMembers,
  removeSpaceMember,
  type SpaceMemberResponse,
  type SpaceResponse,
} from '@voxnexus/api-client';
import { useEffect, useState } from 'react';
import { useAuth } from '../auth';
import { readApiErrorMessage } from '../lib/apiError';
import { Portal } from './ui/Portal';

type Props = {
  open: boolean;
  space: SpaceResponse | null;
  communityId: string;
  onClose: () => void;
};

export function SpaceMembersModal({ open, space, communityId, onClose }: Props) {
  const { session } = useAuth();
  const [members, setMembers] = useState<SpaceMemberResponse[]>([]);
  const [communityMembers, setCommunityMembers] = useState<CommunityMemberResponse[]>([]);
  const [isOwner, setIsOwner] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [selected, setSelected] = useState('');

  const refresh = async () => {
    if (!space) return;
    const [spaceResult, communityResult, communityMeta] = await Promise.all([
      listSpaceMembers({ path: { space_id: space.id } }),
      listCommunityMembers({ path: { community_id: communityId }, query: { limit: 100 } }),
      getCommunity({ path: { community_id: communityId } }),
    ]);
    if (spaceResult.data?.members) {
      setMembers(spaceResult.data.members);
    }
    if (communityResult.data?.items) {
      setCommunityMembers(communityResult.data.items);
    }
    setIsOwner(communityMeta.data?.owner_account_id === session.account.id);
  };

  useEffect(() => {
    if (!open || !space) return;
    setError(null);
    setSelected('');
    void refresh();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, space?.id, communityId, onClose, session.account.id]);

  if (!open || !space) return null;

  const memberIds = new Set(members.map((m) => m.account_id));
  const addable = communityMembers.filter((m) => !memberIds.has(m.account_id));

  const add = async () => {
    if (!selected) {
      setError('Pick a community member to add.');
      return;
    }
    setPending(true);
    setError(null);
    const result = await addSpaceMember({
      path: { space_id: space.id },
      body: { account_id: selected },
    });
    setPending(false);
    if (result.error) {
      setError(readApiErrorMessage(result.error, 'Could not add member.'));
      return;
    }
    setSelected('');
    await refresh();
  };

  const remove = async (accountId: string) => {
    setPending(true);
    setError(null);
    const result = await removeSpaceMember({
      path: { space_id: space.id, account_id: accountId },
    });
    setPending(false);
    if (result.error) {
      setError(readApiErrorMessage(result.error, 'Could not remove member.'));
      return;
    }
    await refresh();
  };

  return (
    <Portal>
      <div className="fixed inset-0 z-[80] grid place-items-center p-4">
        <button
          type="button"
          aria-label="Close"
          className="absolute inset-0 bg-black/55"
          onClick={onClose}
        />
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="space-members-title"
          className="relative z-10 w-full max-w-md rounded-xl border border-line bg-panel p-5 shadow-xl"
        >
          <h2 id="space-members-title" className="mb-1 text-lg font-semibold text-ink">
            {space.name} members
          </h2>
          <p className="mb-4 text-[13px] text-ink-3">
            Restricted spaces are only visible to people listed here.
          </p>

          <ul className="mb-4 max-h-48 space-y-1 overflow-y-auto">
            {members.length === 0 ? (
              <li className="text-[13px] text-ink-4">No members yet.</li>
            ) : (
              members.map((member) => (
                <li
                  key={member.account_id}
                  className="flex items-center gap-2 rounded-md px-2 py-1.5 hover:bg-surface-hover/50"
                >
                  <span className="min-w-0 flex-1 truncate text-[13px] text-ink">
                    {member.display_name}
                  </span>
                  {isOwner && member.account_id !== session.account.id ? (
                    <button
                      type="button"
                      disabled={pending}
                      onClick={() => void remove(member.account_id)}
                      className="text-[12px] text-dnd hover:underline disabled:opacity-50"
                    >
                      Remove
                    </button>
                  ) : null}
                </li>
              ))
            )}
          </ul>

          {isOwner ? (
            <div className="mb-3 flex gap-2">
              <select
                value={selected}
                onChange={(e) => setSelected(e.target.value)}
                className="min-w-0 flex-1 rounded-md border border-line bg-surface px-2 py-1.5 text-[13px] text-ink"
              >
                <option value="">Add community member…</option>
                {addable.map((m) => (
                  <option key={m.account_id} value={m.account_id}>
                    {m.nickname.trim() || m.display_name}
                  </option>
                ))}
              </select>
              <button
                type="button"
                disabled={pending || !selected}
                onClick={() => void add()}
                className="rounded-md bg-accent px-3 py-1.5 text-[13px] font-medium text-app disabled:opacity-50"
              >
                Add
              </button>
            </div>
          ) : null}

          {error ? <p className="mb-2 text-[12px] text-dnd">{error}</p> : null}

          <div className="flex justify-end">
            <button
              type="button"
              onClick={onClose}
              className="rounded-md px-3 py-1.5 text-[13px] text-ink-2 hover:bg-surface-hover/70"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </Portal>
  );
}
