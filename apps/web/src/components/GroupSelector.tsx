import { listSpaces, type SpaceResponse } from '@voxnexus/api-client';
import { ChevronRight, Home, Lock, Plus } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { readApiErrorMessage } from '../lib/apiError';
import { menuFor } from '../lib/menus';
import { useUI } from '../store';
import { CreateSpaceModal } from './CreateSpaceModal';

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export function GroupSelector() {
  const activeCommunity = useUI((s) => s.activeCommunity);
  const active = useUI((s) => s.activeGroup);
  const setGroup = useUI((s) => s.setGroup);
  const collapsed = useUI((s) => s.collapsedSections);
  const toggleSection = useUI((s) => s.toggleSection);
  const openMenu = useUI((s) => s.openMenu);
  const [spaces, setSpaces] = useState<SpaceResponse[]>([]);
  const [createOpen, setCreateOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const isLiveCommunity = UUID_RE.test(activeCommunity);

  const refresh = useCallback(async () => {
    if (!isLiveCommunity) {
      setSpaces([]);
      return;
    }
    const result = await listSpaces({ path: { community_id: activeCommunity } });
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not load spaces.'));
      setSpaces([]);
      return;
    }
    setError(null);
    setSpaces(result.data.spaces);
    if (result.data.spaces.length > 0) {
      const stillSelected = result.data.spaces.some((space) => space.id === active);
      if (!stillSelected) {
        setGroup(result.data.spaces[0].id);
      }
    }
  }, [active, activeCommunity, isLiveCommunity, setGroup]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  if (!isLiveCommunity) {
    return null;
  }

  const section = 'SPACES';
  const isCollapsed = collapsed[section];

  return (
    <div className="border-b border-line/60 px-2 py-2">
      <div className="mb-1.5">
        <div className="flex items-center gap-1 px-1.5 py-1">
          <button
            type="button"
            onClick={() => toggleSection(section)}
            className="group flex min-w-0 flex-1 items-center gap-1 text-left"
          >
            <ChevronRight
              size={11}
              className={`text-ink-4 transition-transform duration-150 ${isCollapsed ? '' : 'rotate-90'}`}
            />
            <span className="kicker group-hover:text-ink-2">{section}</span>
          </button>
          <button
            type="button"
            title="Create space"
            onClick={() => setCreateOpen(true)}
            className="rounded p-0.5 text-ink-4 hover:bg-surface-hover/70 hover:text-ink"
          >
            <Plus size={14} strokeWidth={2} />
          </button>
        </div>
        {!isCollapsed && (
          <div className="mt-0.5 flex flex-col gap-0.5">
            {spaces.length === 0 ? (
              <p className="px-2 py-1.5 text-[12px] text-ink-4">No spaces yet.</p>
            ) : (
              spaces.map((space) => {
                const isActive = active === space.id;
                return (
                  <button
                    key={space.id}
                    type="button"
                    onClick={() => setGroup(space.id)}
                    onContextMenu={(e) => {
                      e.preventDefault();
                      openMenu({ x: e.clientX, y: e.clientY }, menuFor('group', space.name));
                    }}
                    aria-current={isActive}
                    className={`group relative flex items-center gap-2 rounded-md px-2 py-1.5 text-left transition-colors duration-150 ${
                      isActive
                        ? 'bg-surface-active text-ink'
                        : 'text-ink-2 hover:bg-surface-hover/70 hover:text-ink'
                    }`}
                  >
                    {isActive && <span className="tick" />}
                    <Home
                      size={15}
                      strokeWidth={1.9}
                      className={isActive ? 'text-accent' : 'text-ink-3 group-hover:text-ink-2'}
                    />
                    <span className="flex-1 truncate text-[13px] font-medium">{space.name}</span>
                    {space.visibility === 'restricted' ? (
                      <Lock size={11} className="text-ink-4" />
                    ) : null}
                  </button>
                );
              })
            )}
          </div>
        )}
      </div>
      {error ? <p className="px-2 pb-1 text-[11px] text-dnd">{error}</p> : null}
      <CreateSpaceModal
        open={createOpen}
        communityId={activeCommunity}
        onClose={() => setCreateOpen(false)}
        onCreated={(space) => {
          setCreateOpen(false);
          setSpaces((prev) => [...prev, space]);
          setGroup(space.id);
          void refresh();
        }}
      />
    </div>
  );
}
