import {
  type CommunityResponse,
  getCommunity,
  type JoinMode,
  updateCommunity,
} from '@voxnexus/api-client';
import { useEffect, useState } from 'react';
import { readApiErrorMessage } from '../lib/apiError';
import { useUI } from '../store';
import { Portal } from './ui/Portal';

type Props = {
  communityId: string;
};

export function CommunitySettingsModal({ communityId }: Props) {
  const open = useUI((s) => s.communitySettingsOpen);
  const setOpen = useUI((s) => s.setCommunitySettingsOpen);
  const [community, setCommunity] = useState<CommunityResponse | null>(null);
  const [joinMode, setJoinMode] = useState<Exclude<JoinMode, 'application'>>('open');
  const [discoverable, setDiscoverable] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  useEffect(() => {
    if (!open) return;
    setError(null);
    setPending(false);
    void (async () => {
      const result = await getCommunity({ path: { community_id: communityId } });
      if (result.error || !result.data) {
        setError(readApiErrorMessage(result.error, 'Could not load community settings.'));
        setCommunity(null);
        return;
      }
      setCommunity(result.data);
      setJoinMode(result.data.join_mode === 'invite' ? 'invite' : 'open');
      setDiscoverable(result.data.discoverable_on_instance);
    })();
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
                Show in the instance directory when that feature ships. Independent of join mode.
              </span>
            </span>
          </label>

          {error ? <p className="mb-3 text-sm text-[rgb(var(--danger))]">{error}</p> : null}

          <div className="flex justify-end gap-2">
            <button
              type="button"
              onClick={() => setOpen(false)}
              className="rounded-lg px-3 py-2 text-sm text-ink-2 hover:bg-surface-hover"
            >
              Cancel
            </button>
            <button
              type="button"
              disabled={pending || !community}
              onClick={() => void save()}
              className="rounded-lg bg-accent px-3 py-2 text-sm font-medium text-app disabled:opacity-60"
            >
              {pending ? 'Saving…' : 'Save'}
            </button>
          </div>
        </div>
      </div>
    </Portal>
  );
}
