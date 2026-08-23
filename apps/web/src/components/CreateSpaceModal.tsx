import { createSpace, type SpaceResponse } from '@voxnexus/api-client';
import { useEffect, useRef, useState } from 'react';
import { readApiErrorMessage } from '../lib/apiError';
import { Portal } from './ui/Portal';

type Props = {
  open: boolean;
  communityId: string;
  onClose: () => void;
  onCreated: (space: SpaceResponse) => void;
};

export function CreateSpaceModal({ open, communityId, onClose, onCreated }: Props) {
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [topic, setTopic] = useState('');
  const [game, setGame] = useState('');
  const [restricted, setRestricted] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setName('');
      setDescription('');
      setTopic('');
      setGame('');
      setRestricted(false);
      setError(null);
      setPending(false);
      setTimeout(() => inputRef.current?.focus(), 20);
    }
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, onClose]);

  if (!open) return null;

  const submit = async () => {
    const trimmed = name.trim();
    if (!trimmed) {
      setError('Name is required.');
      return;
    }
    setPending(true);
    setError(null);
    const result = await createSpace({
      path: { community_id: communityId },
      body: {
        name: trimmed,
        description: description.trim() || undefined,
        topic: topic.trim() || undefined,
        game: game.trim() || undefined,
        visibility: restricted ? 'restricted' : 'open',
      },
    });
    setPending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not create space.'));
      return;
    }
    onCreated(result.data);
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
          aria-labelledby="create-space-title"
          className="relative w-full max-w-md rounded-2xl border border-line/80 bg-panel p-5 shadow-xl"
        >
          <h2 id="create-space-title" className="mb-1 text-lg font-semibold text-ink">
            Create a space
          </h2>
          <p className="mb-4 text-sm text-ink-3">
            A group inside this community — channels come later.
          </p>
          <label className="mb-3 block text-xs font-medium uppercase tracking-wide text-ink-3">
            Name
            <input
              ref={inputRef}
              value={name}
              maxLength={100}
              onChange={(e) => setName(e.target.value)}
              className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] font-normal normal-case tracking-normal text-ink outline-none focus:border-accent/60"
            />
          </label>
          <label className="mb-3 block text-xs font-medium uppercase tracking-wide text-ink-3">
            Description
            <textarea
              value={description}
              maxLength={2000}
              rows={2}
              onChange={(e) => setDescription(e.target.value)}
              className="mt-1 w-full resize-none rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] font-normal normal-case tracking-normal text-ink outline-none focus:border-accent/60"
            />
          </label>
          <div className="mb-3 grid gap-3 sm:grid-cols-2">
            <label className="block text-xs font-medium uppercase tracking-wide text-ink-3">
              Topic
              <input
                value={topic}
                maxLength={200}
                onChange={(e) => setTopic(e.target.value)}
                className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] font-normal normal-case tracking-normal text-ink outline-none focus:border-accent/60"
              />
            </label>
            <label className="block text-xs font-medium uppercase tracking-wide text-ink-3">
              Game
              <input
                value={game}
                maxLength={100}
                onChange={(e) => setGame(e.target.value)}
                className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] font-normal normal-case tracking-normal text-ink outline-none focus:border-accent/60"
              />
            </label>
          </div>
          <label className="mb-4 flex items-center gap-2 text-[13px] text-ink-2">
            <input
              type="checkbox"
              checked={restricted}
              onChange={(e) => setRestricted(e.target.checked)}
              className="rounded border-line-2"
            />
            Restricted (members need access — enforced later)
          </label>
          {error ? <p className="mb-3 text-[13px] text-dnd">{error}</p> : null}
          <div className="flex justify-end gap-2">
            <button
              type="button"
              onClick={onClose}
              className="rounded-lg px-3 py-2 text-[13px] font-medium text-ink-2 hover:text-ink"
            >
              Cancel
            </button>
            <button
              type="button"
              disabled={pending}
              onClick={() => void submit()}
              className="rounded-lg bg-accent px-3 py-2 text-[13px] font-semibold text-app hover:brightness-110 disabled:opacity-60"
            >
              {pending ? 'Creating…' : 'Create space'}
            </button>
          </div>
        </div>
      </div>
    </Portal>
  );
}
