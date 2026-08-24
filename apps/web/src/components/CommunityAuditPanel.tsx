import { type AuditEventResponse, listAuditEvents } from '@voxnexus/api-client';
import { useCallback, useEffect, useState } from 'react';
import { readApiErrorMessage } from '../lib/apiError';

type Props = {
  communityId: string;
  canView: boolean;
};

export function CommunityAuditPanel({ communityId, canView }: Props) {
  const [items, setItems] = useState<AuditEventResponse[]>([]);
  const [hasMore, setHasMore] = useState(false);
  const [actionFilter, setActionFilter] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  const refresh = useCallback(async () => {
    if (!canView) return;
    setPending(true);
    setError(null);
    const result = await listAuditEvents({
      path: { community_id: communityId },
      query: {
        limit: 30,
        ...(actionFilter.trim() ? { action: actionFilter.trim() } : {}),
      },
    });
    setPending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not load audit log.'));
      setItems([]);
      return;
    }
    setItems(result.data.items);
    setHasMore(result.data.has_more);
  }, [actionFilter, canView, communityId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const loadMore = async () => {
    const after = items[items.length - 1]?.id;
    if (!after) return;
    setPending(true);
    setError(null);
    const result = await listAuditEvents({
      path: { community_id: communityId },
      query: {
        limit: 30,
        after,
        ...(actionFilter.trim() ? { action: actionFilter.trim() } : {}),
      },
    });
    setPending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not load more audit events.'));
      return;
    }
    setItems((prev) => [...prev, ...result.data!.items]);
    setHasMore(result.data.has_more);
  };

  if (!canView) {
    return <p className="text-sm text-ink-3">You need View Audit Log permission to see this.</p>;
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-3 overflow-hidden px-8 py-6 pr-16">
      <div>
        <h2 className="text-lg font-semibold text-ink">Audit Log</h2>
        <p className="text-sm text-ink-3">Recent community changes (newest first).</p>
      </div>
      <label className="block text-xs font-medium uppercase tracking-wide text-ink-3">
        Action filter
        <input
          value={actionFilter}
          onChange={(e) => setActionFilter(e.target.value)}
          placeholder="e.g. role.create"
          className="mt-1 w-full max-w-xs rounded-lg border border-line-2/80 bg-surface px-3 py-2 text-sm normal-case text-ink outline-none focus:border-accent/50"
        />
      </label>
      {error ? <p className="text-sm text-dnd">{error}</p> : null}
      <div className="min-h-0 flex-1 overflow-y-auto rounded-lg border border-line/70">
        {items.length === 0 && !pending ? (
          <p className="p-4 text-sm text-ink-4">No audit events yet.</p>
        ) : (
          <ul className="divide-y divide-line/60">
            {items.map((event) => (
              <li key={event.id} className="px-3 py-2.5">
                <div className="flex flex-wrap items-baseline gap-x-2 gap-y-0.5">
                  <span className="font-mono text-[11px] text-accent">{event.action}</span>
                  <span className="text-[11px] text-ink-4">
                    {new Date(event.created_at).toLocaleString()}
                  </span>
                </div>
                <p className="text-sm text-ink">{event.summary}</p>
                {event.actor_account_id ? (
                  <p className="font-mono text-[10px] text-ink-4">
                    actor {event.actor_account_id.slice(0, 8)}…
                  </p>
                ) : null}
              </li>
            ))}
          </ul>
        )}
      </div>
      {hasMore ? (
        <button
          type="button"
          disabled={pending}
          onClick={() => void loadMore()}
          className="self-start rounded-lg border border-line/80 px-3 py-1.5 text-sm text-ink-2 hover:bg-surface-hover disabled:opacity-50"
        >
          {pending ? 'Loading…' : 'Load more'}
        </button>
      ) : null}
    </div>
  );
}
