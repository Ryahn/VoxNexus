import { joinCommunity } from '@voxnexus/api-client';
import { useEffect, useRef, useState } from 'react';
import { useUI } from '../store';
import { Portal } from './ui/Portal';

type Props = {
  onJoined: (id: string) => void;
};

export function JoinCommunityModal({ onJoined }: Props) {
  const open = useUI((s) => s.joinCommunityOpen);
  const setOpen = useUI((s) => s.setJoinCommunityOpen);
  const [communityId, setCommunityId] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setCommunityId('');
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
    const trimmed = communityId.trim();
    if (!trimmed) {
      setError('Community id is required.');
      return;
    }
    setPending(true);
    setError(null);
    const result = await joinCommunity({ path: { community_id: trimmed } });
    setPending(false);
    if (result.error || !result.data) {
      const message =
        result.error && typeof result.error === 'object' && 'message' in result.error
          ? String((result.error as { message: string }).message)
          : 'Could not join community.';
      setError(message);
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
            Enter a community id for an open community. Invite links come later.
          </p>
          <label className="mb-3 block text-xs font-medium uppercase tracking-wide text-ink-3">
            Community id
            <input
              ref={inputRef}
              value={communityId}
              onChange={(e) => setCommunityId(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') void submit();
              }}
              className="mt-1.5 w-full rounded-lg border border-line-2/60 bg-input px-3 py-2 font-mono text-sm text-ink outline-none focus:border-accent/50"
              placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
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
