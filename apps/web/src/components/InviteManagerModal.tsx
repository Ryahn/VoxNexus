import {
  createCommunityInvite,
  type InviteResponse,
  listCommunityInvites,
  revokeCommunityInvite,
  updateCommunityInvite,
} from '@voxnexus/api-client';
import { useEffect, useState } from 'react';
import { useUI } from '../store';
import { Portal } from './ui/Portal';

type Props = {
  communityId: string;
};

type ExpireUnit = 'hours' | 'days' | 'months' | 'never';

const UNIT_MAX: Record<Exclude<ExpireUnit, 'never'>, number> = {
  hours: 24,
  days: 14,
  months: 3,
};

export function InviteManagerModal({ communityId }: Props) {
  const open = useUI((s) => s.inviteManagerOpen);
  const setOpen = useUI((s) => s.setInviteManagerOpen);
  const [invites, setInvites] = useState<InviteResponse[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [expireUnit, setExpireUnit] = useState<ExpireUnit>('days');
  const [expireValue, setExpireValue] = useState(7);
  const [maxUses, setMaxUses] = useState('');
  const [unlimitedUses, setUnlimitedUses] = useState(true);

  const refresh = async () => {
    const result = await listCommunityInvites({ path: { community_id: communityId } });
    if (result.data?.invites) {
      setInvites(result.data.invites);
    }
  };

  useEffect(() => {
    if (!open) return;
    setError(null);
    setExpireUnit('days');
    setExpireValue(7);
    setMaxUses('');
    setUnlimitedUses(true);
    void refresh();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setOpen(false);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, communityId, setOpen]);

  useEffect(() => {
    if (expireUnit === 'never') return;
    const max = UNIT_MAX[expireUnit];
    if (expireValue > max) setExpireValue(max);
    if (expireValue < 1) setExpireValue(1);
  }, [expireUnit, expireValue]);

  if (!open) return null;

  const create = async () => {
    setPending(true);
    setError(null);

    let uses: number | undefined;
    if (!unlimitedUses) {
      const parsed = Number.parseInt(maxUses, 10);
      if (!Number.isFinite(parsed) || parsed < 1 || parsed > 1000) {
        setPending(false);
        setError('Uses must be between 1 and 1000, or unlimited.');
        return;
      }
      uses = parsed;
    }

    const result = await createCommunityInvite({
      path: { community_id: communityId },
      body: {
        max_uses: uses,
        expire_after:
          expireUnit === 'never'
            ? undefined
            : {
                unit: expireUnit,
                value: expireValue,
              },
      },
    });
    setPending(false);
    if (result.error || !result.data) {
      const message =
        result.error && typeof result.error === 'object' && 'message' in result.error
          ? String((result.error as { message: string }).message)
          : 'Could not create invite.';
      setError(message);
      return;
    }
    await refresh();
  };

  const togglePause = async (invite: InviteResponse) => {
    await updateCommunityInvite({
      path: { community_id: communityId, invite_id: invite.id },
      body: { paused: !invite.paused },
    });
    await refresh();
  };

  const revoke = async (invite: InviteResponse) => {
    await revokeCommunityInvite({
      path: { community_id: communityId, invite_id: invite.id },
    });
    await refresh();
  };

  const unitMax = expireUnit === 'never' ? 1 : UNIT_MAX[expireUnit];

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
          aria-labelledby="invite-manager-title"
          className="relative w-full max-w-lg rounded-2xl border border-line/80 bg-panel p-5 shadow-xl"
        >
          <h2 id="invite-manager-title" className="mb-1 text-lg font-semibold text-ink">
            Invite people
          </h2>
          <p className="mb-4 text-sm text-ink-3">
            Configure expiry and uses, then generate a code.
          </p>

          <div className="mb-4 grid gap-3 rounded-xl border border-line/60 bg-surface/30 p-3">
            <label className="block text-xs font-medium uppercase tracking-wide text-ink-3">
              Expires after
              <div className="mt-1.5 flex gap-2">
                <input
                  type="number"
                  min={1}
                  max={unitMax}
                  disabled={expireUnit === 'never'}
                  value={expireUnit === 'never' ? '' : expireValue}
                  onChange={(e) => setExpireValue(Number(e.target.value) || 1)}
                  className="w-20 rounded-lg border border-line-2/60 bg-input px-2 py-1.5 text-sm text-ink outline-none focus:border-accent/50 disabled:opacity-50"
                />
                <select
                  value={expireUnit}
                  onChange={(e) => setExpireUnit(e.target.value as ExpireUnit)}
                  className="flex-1 rounded-lg border border-line-2/60 bg-input px-2 py-1.5 text-sm text-ink outline-none focus:border-accent/50"
                >
                  <option value="hours">Hours (max 24)</option>
                  <option value="days">Days (max 14)</option>
                  <option value="months">Months (max 3)</option>
                  <option value="never">Never</option>
                </select>
              </div>
            </label>

            <label className="block text-xs font-medium uppercase tracking-wide text-ink-3">
              Max uses
              <div className="mt-1.5 flex items-center gap-2">
                <input
                  type="number"
                  min={1}
                  max={1000}
                  disabled={unlimitedUses}
                  value={unlimitedUses ? '' : maxUses}
                  onChange={(e) => setMaxUses(e.target.value)}
                  placeholder="1–1000"
                  className="w-28 rounded-lg border border-line-2/60 bg-input px-2 py-1.5 text-sm text-ink outline-none focus:border-accent/50 disabled:opacity-50"
                />
                <label className="flex items-center gap-1.5 text-xs normal-case tracking-normal text-ink-2">
                  <input
                    type="checkbox"
                    checked={unlimitedUses}
                    onChange={(e) => setUnlimitedUses(e.target.checked)}
                  />
                  Unlimited
                </label>
              </div>
            </label>

            <div className="flex justify-end">
              <button
                type="button"
                disabled={pending}
                onClick={() => void create()}
                className="rounded-lg bg-accent px-3 py-2 text-sm font-medium text-app disabled:opacity-60"
              >
                {pending ? 'Creating…' : 'Generate invite'}
              </button>
            </div>
          </div>

          {error ? <p className="mb-3 text-sm text-danger">{error}</p> : null}
          <ul className="max-h-60 space-y-2 overflow-y-auto">
            {invites.length === 0 ? (
              <li className="text-sm text-ink-3">No active invites yet.</li>
            ) : (
              invites.map((invite) => (
                <li
                  key={invite.id}
                  className="flex items-center gap-2 rounded-lg border border-line/60 bg-surface/40 px-3 py-2"
                >
                  <code className="min-w-0 flex-1 truncate font-mono text-xs text-ink">
                    {invite.code}
                  </code>
                  <span className="shrink-0 font-mono text-3xs text-ink-3">
                    {invite.uses}
                    {invite.max_uses != null ? `/${invite.max_uses}` : ''} uses
                    {invite.expires_at
                      ? ` · exp ${new Date(invite.expires_at).toLocaleString()}`
                      : ' · no expiry'}
                    {invite.paused ? ' · paused' : ''}
                  </span>
                  <button
                    type="button"
                    onClick={() => void togglePause(invite)}
                    className="rounded px-2 py-1 text-xs text-ink-2 hover:bg-surface-hover"
                  >
                    {invite.paused ? 'Resume' : 'Pause'}
                  </button>
                  <button
                    type="button"
                    onClick={() => void revoke(invite)}
                    className="rounded px-2 py-1 text-xs text-danger hover:bg-surface-hover"
                  >
                    Revoke
                  </button>
                </li>
              ))
            )}
          </ul>
          <div className="mt-4 flex justify-end">
            <button
              type="button"
              onClick={() => setOpen(false)}
              className="rounded-lg px-3 py-2 text-sm text-ink-2 hover:bg-surface-hover"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </Portal>
  );
}
