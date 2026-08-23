import {
  type CategoryResponse,
  type ChannelResponse,
  getCommunity,
  listCategories,
  listChannels,
  reorderCategories,
} from '@voxnexus/api-client';
import { Plus } from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useAuth } from '../auth';
import { apiChannelToUi } from '../lib/apiChannel';
import { readApiErrorMessage } from '../lib/apiError';
import { useUI } from '../store';
import type { Category, Channel } from '../types';
import { ChannelCategory } from './ChannelCategory';
import { CreateCategoryModal } from './CreateCategoryModal';
import { CreateChannelModal } from './CreateChannelModal';

type Props = {
  communityId: string;
  spaceId: string;
  spaceName: string;
};

function toSidebarCategory(cat: CategoryResponse, spaceId: string): Category {
  return { id: cat.id, groupId: spaceId, name: cat.name };
}

export function LiveCategoryList({ communityId, spaceId, spaceName }: Props) {
  const { session } = useAuth();
  const setChannel = useUI((s) => s.setChannel);
  const [categories, setCategories] = useState<CategoryResponse[]>([]);
  const [channels, setChannels] = useState<ChannelResponse[]>([]);
  const [isOwner, setIsOwner] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [createCategoryOpen, setCreateCategoryOpen] = useState(false);
  const [createChannelFor, setCreateChannelFor] = useState<string | null>(null);
  const dragId = useRef<string | null>(null);

  const refresh = useCallback(async () => {
    const [listResult, channelResult, communityResult] = await Promise.all([
      listCategories({
        path: { community_id: communityId },
        query: { space_id: spaceId },
      }),
      listChannels({
        path: { community_id: communityId },
        query: { space_id: spaceId },
      }),
      getCommunity({ path: { community_id: communityId } }),
    ]);
    if (listResult.error || !listResult.data) {
      setError(readApiErrorMessage(listResult.error, 'Could not load categories.'));
      setCategories([]);
      setChannels([]);
      return;
    }
    if (channelResult.error || !channelResult.data) {
      setError(readApiErrorMessage(channelResult.error, 'Could not load channels.'));
      setCategories(listResult.data.categories);
      setChannels([]);
      return;
    }
    setError(null);
    setCategories(listResult.data.categories);
    setChannels(channelResult.data.channels);
    setIsOwner(communityResult.data?.owner_account_id === session.account.id);
  }, [communityId, spaceId, session.account.id]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const reorder = async (ordered: CategoryResponse[]) => {
    setCategories(ordered);
    const result = await reorderCategories({
      path: { community_id: communityId },
      body: { category_ids: ordered.map((cat) => cat.id) },
    });
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not reorder categories.'));
      await refresh();
      return;
    }
    setCategories(result.data.categories);
  };

  const handleDrop = (targetId: string) => {
    const sourceId = dragId.current;
    dragId.current = null;
    if (!sourceId || sourceId === targetId) return;
    const ids = categories.map((cat) => cat.id);
    const from = ids.indexOf(sourceId);
    const to = ids.indexOf(targetId);
    if (from < 0 || to < 0) return;
    const next = [...categories];
    const [moved] = next.splice(from, 1);
    next.splice(to, 0, moved);
    void reorder(next);
  };

  const channelsForCategory = (categoryId: string): Channel[] =>
    channels.filter((ch) => ch.category_id === categoryId).map((ch) => apiChannelToUi(ch, spaceId));

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center justify-between px-3 pb-1 pt-2">
        <span className="font-sans text-[11px] font-semibold uppercase tracking-[0.12em] text-ink-3">
          {spaceName}
        </span>
        {isOwner ? (
          <button
            type="button"
            title="Create category"
            onClick={() => setCreateCategoryOpen(true)}
            className="rounded p-0.5 text-ink-4 hover:bg-surface-hover/70 hover:text-ink"
          >
            <Plus size={14} strokeWidth={2} />
          </button>
        ) : null}
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-3">
        {categories.length === 0 ? (
          <p className="px-2 py-2 text-[12px] text-ink-4">
            No categories yet. Create one to add channels.
          </p>
        ) : (
          categories.map((cat) => {
            const sidebarCat = toSidebarCategory(cat, spaceId);
            return (
              <div
                key={cat.id}
                draggable={isOwner}
                onDragStart={() => {
                  dragId.current = cat.id;
                }}
                onDragOver={(e) => {
                  if (isOwner) e.preventDefault();
                }}
                onDrop={() => {
                  if (isOwner) handleDrop(cat.id);
                }}
                className={isOwner ? 'cursor-grab active:cursor-grabbing' : undefined}
              >
                <ChannelCategory
                  category={sidebarCat}
                  channels={channelsForCategory(cat.id)}
                  onAddChannel={isOwner ? () => setCreateChannelFor(cat.id) : undefined}
                />
              </div>
            );
          })
        )}
      </div>
      {error ? <p className="px-2 pb-2 text-[11px] text-dnd">{error}</p> : null}
      <CreateCategoryModal
        open={createCategoryOpen}
        communityId={communityId}
        spaceId={spaceId}
        onClose={() => setCreateCategoryOpen(false)}
        onCreated={() => void refresh()}
      />
      {createChannelFor ? (
        <CreateChannelModal
          open
          communityId={communityId}
          spaceId={spaceId}
          categoryId={createChannelFor}
          onClose={() => setCreateChannelFor(null)}
          onCreated={(channelId) => {
            setCreateChannelFor(null);
            setChannel(channelId);
            void refresh();
          }}
        />
      ) : null}
    </div>
  );
}
