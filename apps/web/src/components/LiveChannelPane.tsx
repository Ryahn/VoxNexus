import { archiveChannel, getChannel, getCommunity } from '@voxnexus/api-client';
import { Archive } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useAuth } from '../auth';
import { apiTypeToUi } from '../lib/apiChannel';
import { readApiErrorMessage } from '../lib/apiError';
import { channelMeta } from '../lib/channelMeta';
import { useUI } from '../store';

type Props = {
  channelId: string;
  onArchived?: () => void;
};

export function LiveChannelPane({ channelId, onArchived }: Props) {
  const { session } = useAuth();
  const setChannel = useUI((s) => s.setChannel);
  const [name, setName] = useState('Channel');
  const [topic, setTopic] = useState('');
  const [apiType, setApiType] = useState('text');
  const [isOwner, setIsOwner] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  const refresh = useCallback(async () => {
    const result = await getChannel({ path: { channel_id: channelId } });
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not load channel.'));
      return;
    }
    const channel = result.data;
    setName(channel.name);
    setTopic(channel.topic);
    setApiType(channel.type);
    setError(null);

    const communityResult = await getCommunity({
      path: { community_id: channel.community_id },
    });
    setIsOwner(communityResult.data?.owner_account_id === session.account.id);
  }, [channelId, session.account.id]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const uiType = apiTypeToUi(apiType);
  const { Icon, label } = channelMeta[uiType] ?? channelMeta.text;
  const isText = apiType === 'text';

  const archive = async () => {
    if (!isOwner || pending) return;
    setPending(true);
    const result = await archiveChannel({ path: { channel_id: channelId } });
    setPending(false);
    if (result.error) {
      setError(readApiErrorMessage(result.error, 'Could not archive channel.'));
      return;
    }
    onArchived?.();
    setChannel(channelId);
  };

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <header className="flex h-12 shrink-0 items-center gap-2 border-b border-line/70 px-4">
        <Icon size={18} className="shrink-0 text-ink-3" strokeWidth={1.9} />
        <div className="min-w-0 flex-1">
          <h1 className="truncate text-[15px] font-semibold text-ink">{name}</h1>
          {topic ? <p className="truncate text-xs text-ink-3">{topic}</p> : null}
        </div>
        {isOwner ? (
          <button
            type="button"
            title="Archive channel"
            disabled={pending}
            onClick={() => void archive()}
            className="rounded p-1.5 text-ink-3 hover:bg-surface-hover hover:text-ink disabled:opacity-50"
          >
            <Archive size={16} />
          </button>
        ) : null}
      </header>
      <div className="flex min-h-0 flex-1 flex-col items-center justify-center px-6 text-center">
        {error ? (
          <p className="text-sm text-dnd">{error}</p>
        ) : isText ? (
          <>
            <p className="text-sm font-medium text-ink-2">No messages yet</p>
            <p className="mt-1 max-w-sm text-sm text-ink-3">
              This is the start of <span className="font-medium text-ink-2">#{name}</span>.
              Messaging arrives in a later milestone.
            </p>
          </>
        ) : (
          <>
            <p className="text-sm font-medium text-ink-2">{label} channels</p>
            <p className="mt-1 max-w-sm text-sm text-ink-3">
              {label} features are coming in a later milestone.
            </p>
          </>
        )}
      </div>
    </div>
  );
}
