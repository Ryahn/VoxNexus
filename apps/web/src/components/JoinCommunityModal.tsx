import { acceptInvite, joinCommunity } from '@voxnexus/api-client';
import { useEffect, useRef, useState } from 'react';
import { readApiErrorMessage } from '../lib/apiError';
import { useUI } from '../store';
import { Portal } from './ui/Portal';

type Props = {
  onJoined: (id: string) => void;
};

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export function JoinCommunityModal({ onJoined }: Props) {
  const open = useUI((s) => s.joinCommunityOpen);
  const setOpen = useUI((s) => s.setJoinCommunityOpen);
  const [value, setValue] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setValue('');
      setError(null);
      setPending(false);
      setTimeout(() => inputRef.current?.focus(), 20);
    }
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setOpen(false);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, setOpen]);

  if (!open) return null;

  const submit = async () => {
    const trimmed = value.trim();
    if (!trimmed) {
      setError('Community id or invite code is required.');
      return;
    }
    setPending(true);
    setError(null);
    if (UUID_RE.test(trimmed)) {
      const result = await joinCommunity({ path: { community_id: trimmed } });
      setPending(false);
      if (result.error || !result.data) {
        setError(
          readApiErrorMessage(
            result.error,
            'Could not join. Invite-only communities require an invite code.',
          ),
        );
        return;
      }
      setOpen(false);
      onJoined(result.data.community_id);
      return;
    }

    const result = await acceptInvite({ path: { code: trimmed } });
    setPending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not accept invite.'));
      return;
    }
    setOpen(false);
    onJoined(result.data.community_id);
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
          aria-labelledby="join-community-title"
          className="relative w-full max-w-md rounded-2xl border border-line/80 bg-panel p-5 shadow-xl"
        >
          <h2 id="join-community-title" className="mb-1 text-lg font-semibold text-ink">
            Join a community
          </h2>
          <p className="mb-4 text-sm text-ink-3">
            Prefer an invite code. A community id only works when the community is open.
          </p>
          <label className="mb-3 block text-xs font-medium uppercase tracking-wide text-ink-3">
            Invite code or community id
            <input
              ref={inputRef}
              value={value}
              onChange={(e) => setValue(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') void submit();
              }}
              className="mt-1.5 w-full rounded-lg border border-line-2/60 bg-input px-3 py-2 font-mono text-sm text-ink outline-none focus:border-accent/50"
              placeholder="invite code or uuid"
            />
          </label>
          {error ? <p className="mb-3 text-sm text-danger">{error}</p> : null}
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
              disabled={pending}
              onClick={() => void submit()}
              className="rounded-lg bg-accent px-3 py-2 text-sm font-medium text-app disabled:opacity-60"
            >
              {pending ? 'Joining…' : 'Join'}
            </button>
          </div>
        </div>
      </div>
    </Portal>
  );
}
