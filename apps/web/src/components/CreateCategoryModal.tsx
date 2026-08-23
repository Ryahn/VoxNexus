import { createCategory } from '@voxnexus/api-client';
import { useEffect, useRef, useState } from 'react';
import { readApiErrorMessage } from '../lib/apiError';
import { Portal } from './ui/Portal';

type Props = {
  open: boolean;
  communityId: string;
  spaceId: string;
  onClose: () => void;
  onCreated: () => void;
};

export function CreateCategoryModal({ open, communityId, spaceId, onClose, onCreated }: Props) {
  const [name, setName] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setName('');
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
    const result = await createCategory({
      path: { community_id: communityId },
      body: { name: trimmed, space_id: spaceId },
    });
    setPending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not create category.'));
      return;
    }
    onClose();
    onCreated();
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
          aria-labelledby="create-category-title"
          className="relative w-full max-w-sm rounded-2xl border border-line/80 bg-panel p-5 shadow-xl"
        >
          <h2 id="create-category-title" className="mb-1 text-lg font-semibold text-ink">
            New category
          </h2>
          <p className="mb-4 text-sm text-ink-3">Groups channels in this space.</p>
          <label className="mb-4 block text-xs font-medium uppercase tracking-wide text-ink-3">
            Name
            <input
              ref={inputRef}
              value={name}
              onChange={(e) => setName(e.target.value)}
              maxLength={100}
              onKeyDown={(e) => {
                if (e.key === 'Enter') void submit();
              }}
              className="mt-1 w-full rounded-lg border border-line-2/80 bg-surface px-3 py-2 text-sm text-ink outline-none focus:border-accent/50"
            />
          </label>
          {error ? <p className="mb-3 text-sm text-[rgb(var(--danger))]">{error}</p> : null}
          <div className="flex justify-end gap-2">
            <button
              type="button"
              onClick={onClose}
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
