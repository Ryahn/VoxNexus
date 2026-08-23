import { createCommunity } from '@voxnexus/api-client';
import { useEffect, useRef, useState } from 'react';
import { useUI } from '../store';
import { Portal } from './ui/Portal';

type Props = {
  onCreated: (id: string) => void;
};

export function CreateCommunityModal({ onCreated }: Props) {
  const open = useUI((s) => s.createCommunityOpen);
  const setOpen = useUI((s) => s.setCreateCommunityOpen);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setName('');
      setDescription('');
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
    const trimmed = name.trim();
    if (!trimmed) {
      setError('Name is required.');
      return;
    }
    setPending(true);
    setError(null);
    const result = await createCommunity({
      body: {
        name: trimmed,
        description: description.trim() || undefined,
      },
    });
    setPending(false);
    if (result.error || !result.data) {
      const message =
        result.error && typeof result.error === 'object' && 'message' in result.error
          ? String((result.error as { message: string }).message)
          : 'Could not create community.';
      setError(message);
      return;
    }
    setOpen(false);
    onCreated(result.data.id);
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
          aria-labelledby="create-community-title"
          className="relative w-full max-w-md rounded-2xl border border-line/80 bg-panel p-5 shadow-xl"
        >
          <h2 id="create-community-title" className="mb-1 text-lg font-semibold text-ink">
            Create a community
          </h2>
          <p className="mb-4 text-sm text-ink-3">
            A space for your group — channels and roles come next.
          </p>
          <label className="mb-3 block text-xs font-medium uppercase tracking-wide text-ink-3">
            Name
            <input
              ref={inputRef}
              value={name}
              onChange={(e) => setName(e.target.value)}
              maxLength={100}
              className="mt-1 w-full rounded-lg border border-line-2/80 bg-surface px-3 py-2 text-sm text-ink outline-none focus:border-accent/50"
            />
          </label>
          <label className="mb-4 block text-xs font-medium uppercase tracking-wide text-ink-3">
            Description
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              maxLength={2000}
              rows={3}
              className="mt-1 w-full resize-none rounded-lg border border-line-2/80 bg-surface px-3 py-2 text-sm text-ink outline-none focus:border-accent/50"
            />
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
              disabled={pending}
              onClick={() => void submit()}
              className="rounded-lg bg-accent px-3 py-2 text-sm font-medium text-app disabled:opacity-60"
            >
              {pending ? 'Creating…' : 'Create'}
            </button>
          </div>
        </div>
      </div>
    </Portal>
  );
}
