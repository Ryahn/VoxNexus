import { createChannel } from '@voxnexus/api-client';
import { useEffect, useRef, useState } from 'react';
import { uiTypeToApi } from '../lib/apiChannel';
import { readApiErrorMessage } from '../lib/apiError';
import type { ChannelType } from '../types';
import { Portal } from './ui/Portal';

const CREATE_TYPES: { value: ChannelType; label: string }[] = [
  { value: 'text', label: 'Text' },
  { value: 'voice', label: 'Voice' },
  { value: 'announcement', label: 'Announcement' },
  { value: 'forum', label: 'Forum' },
  { value: 'calendar', label: 'Calendar' },
  { value: 'events', label: 'Scheduling' },
  { value: 'docs', label: 'Docs' },
  { value: 'tasks', label: 'Tasks' },
  { value: 'media', label: 'Media' },
  { value: 'stream', label: 'Streaming' },
];

type Props = {
  open: boolean;
  communityId: string;
  spaceId: string;
  categoryId: string;
  onClose: () => void;
  onCreated: (channelId: string) => void;
};

export function CreateChannelModal({
  open,
  communityId,
  spaceId,
  categoryId,
  onClose,
  onCreated,
}: Props) {
  const [name, setName] = useState('');
  const [channelType, setChannelType] = useState<ChannelType>('text');
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setName('');
      setChannelType('text');
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
    const result = await createChannel({
      path: { community_id: communityId },
      body: {
        name: trimmed,
        type: uiTypeToApi(channelType),
        space_id: spaceId,
        category_id: categoryId,
      },
    });
    setPending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not create channel.'));
      return;
    }
    onClose();
    onCreated(result.data.id);
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
          aria-labelledby="create-channel-title"
          className="relative w-full max-w-sm rounded-2xl border border-line/80 bg-panel p-5 shadow-xl"
        >
          <h2 id="create-channel-title" className="mb-1 text-lg font-semibold text-ink">
            New channel
          </h2>
          <p className="mb-4 text-sm text-ink-3">Shell only — messaging comes later.</p>
          <label className="mb-3 block text-xs font-medium uppercase tracking-wide text-ink-3">
            Type
            <select
              value={channelType}
              onChange={(e) => setChannelType(e.target.value as ChannelType)}
              className="mt-1 w-full rounded-lg border border-line-2/80 bg-surface px-3 py-2 text-sm text-ink outline-none focus:border-accent/50"
            >
              {CREATE_TYPES.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </label>
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
