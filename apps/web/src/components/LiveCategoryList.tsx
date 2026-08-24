import {
  type CategoryResponse,
  type ChannelResponse,
  getCommunity,
  listCategories,
  listChannels,
  reorderCategories,
} from '@voxnexus/api-client';
import { FolderPlus, Plus } from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useAuth } from '../auth';
import { apiChannelToUi } from '../lib/apiChannel';
import { readApiErrorMessage } from '../lib/apiError';
import { useUI } from '../store';
import type { Category, Channel } from '../types';
import { ChannelCategory } from './ChannelCategory';
import { ChannelItem } from './ChannelItem';
import { ChannelPermissionsModal } from './ChannelPermissionsModal';
import { CreateCategoryModal } from './CreateCategoryModal';
import { CreateChannelModal } from './CreateChannelModal';
import { ViewAsBar } from './ViewAsBar';

type Props = {
  communityId: string;
  spaceId: string;
  spaceName: string;
};

/** `undefined` = closed; `null` = uncategorized; string = category id. */
type CreateChannelTarget = string | null | undefined;

function toSidebarCategory(cat: CategoryResponse, spaceId: string): Category {
  return { id: cat.id, groupId: spaceId, name: cat.name };
}

export function LiveCategoryList({ communityId, spaceId, spaceName }: Props) {
  const { session } = useAuth();
  const setChannel = useUI((s) => s.setChannel);
  const viewAs = useUI((s) => s.viewAs);
  const [categories, setCategories] = useState<CategoryResponse[]>([]);
  const [channels, setChannels] = useState<ChannelResponse[]>([]);
  const [isOwner, setIsOwner] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [createCategoryOpen, setCreateCategoryOpen] = useState(false);
  const [createChannelFor, setCreateChannelFor] = useState<CreateChannelTarget>(undefined);
  const [simulatedIds, setSimulatedIds] = useState<string[] | null>(null);
  const [simulatedLabel, setSimulatedLabel] = useState<string | null>(null);
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

  useEffect(() => {
    if (!viewAs) {
      setSimulatedIds(null);
      setSimulatedLabel(null);
    }
  }, [viewAs]);

  const reorder = async (ordered: CategoryResponse[]) => {
    if (viewAs) return;
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
    if (viewAs) return;
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

  const displayChannels =
    simulatedIds === null ? channels : channels.filter((ch) => simulatedIds.includes(ch.id));

  const looseChannels: Channel[] = displayChannels
    .filter((ch) => ch.category_id == null)
    .map((ch) => apiChannelToUi(ch, spaceId));

  const channelsForCategory = (categoryId: string): Channel[] =>
    displayChannels
      .filter((ch) => ch.category_id === categoryId)
      .map((ch) => apiChannelToUi(ch, spaceId));

  const canEditStructure = isOwner && !viewAs;
  const isEmpty = categories.length === 0 && displayChannels.length === 0;

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <ViewAsBar
        communityId={communityId}
        spaceId={spaceId}
        canManage={isOwner}
        onSimulatedChannels={(ids, label) => {
          setSimulatedIds(ids);
          setSimulatedLabel(label);
        }}
      />
      {viewAs && simulatedLabel ? (
        <div className="mx-2 mb-1 rounded border border-accent/30 bg-accent/10 px-2 py-1 text-[11px] text-accent">
          Channel list as <strong className="font-semibold">{simulatedLabel}</strong>
        </div>
      ) : null}
      <div className="flex items-center justify-between px-3 pb-1 pt-2">
        <span className="font-sans text-[11px] font-semibold uppercase tracking-[0.12em] text-ink-3">
          {spaceName}
        </span>
        {canEditStructure ? (
          <div className="flex items-center gap-0.5">
            <button
              type="button"
              title="Create channel"
              onClick={() => setCreateChannelFor(null)}
              className="rounded p-0.5 text-ink-4 hover:bg-surface-hover/70 hover:text-ink"
            >
              <Plus size={14} strokeWidth={2} />
            </button>
            <button
              type="button"
              title="Create category"
              onClick={() => setCreateCategoryOpen(true)}
              className="rounded p-0.5 text-ink-4 hover:bg-surface-hover/70 hover:text-ink"
            >
              <FolderPlus size={14} strokeWidth={2} />
            </button>
          </div>
        ) : null}
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-3">
        {isEmpty ? (
          <p className="px-2 py-2 text-[12px] text-ink-4">
            No channels yet. Add a channel here, or create a category to group them.
          </p>
        ) : (
          <>
            {looseChannels.length > 0 ? (
              <div className="mb-1.5 flex flex-col gap-px pl-1">
                {looseChannels.map((ch) => (
                  <ChannelItem
                    key={ch.id}
                    channel={ch}
                    communityId={communityId}
                    canManage={canEditStructure}
                  />
                ))}
              </div>
            ) : null}
            {categories.map((cat) => {
              const sidebarCat = toSidebarCategory(cat, spaceId);
              return (
                <div
                  key={cat.id}
                  draggable={canEditStructure}
                  onDragStart={() => {
                    dragId.current = cat.id;
                  }}
                  onDragOver={(e) => {
                    if (canEditStructure) e.preventDefault();
                  }}
                  onDrop={() => {
                    if (canEditStructure) handleDrop(cat.id);
                  }}
                  className={canEditStructure ? 'cursor-grab active:cursor-grabbing' : undefined}
                >
                  <ChannelCategory
                    category={sidebarCat}
                    channels={channelsForCategory(cat.id)}
                    onAddChannel={canEditStructure ? () => setCreateChannelFor(cat.id) : undefined}
                    communityId={communityId}
                    canManage={canEditStructure}
                  />
                </div>
              );
            })}
          </>
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
      {createChannelFor !== undefined ? (
        <CreateChannelModal
          open
          communityId={communityId}
          spaceId={spaceId}
          categoryId={createChannelFor}
          onClose={() => setCreateChannelFor(undefined)}
          onCreated={(channelId) => {
            setCreateChannelFor(undefined);
            setChannel(channelId);
            void refresh();
          }}
        />
      ) : null}
      <ChannelPermissionsModal />
    </div>
  );
}
