import { ChevronDown, Plus } from 'lucide-react';
import { useUI } from '../store';
import type { Category, Channel } from '../types';
import { ChannelItem } from './ChannelItem';

export function ChannelCategory({
  category,
  channels,
  onAddChannel,
}: {
  category: Category;
  channels: Channel[];
  onAddChannel?: () => void;
}) {
  const collapsed = useUI((s) => s.collapsedCats[category.id]);
  const toggle = useUI((s) => s.toggleCat);

  // when collapsed, still surface channels with unread/mentions
  const visible = collapsed ? channels.filter((c) => c.unread || (c.mentions ?? 0) > 0) : channels;

  return (
    <section className="mb-1.5">
      <div className="group flex items-center gap-1 px-1 pb-0.5 pt-1">
        <button
          type="button"
          onClick={() => toggle(category.id)}
          className="flex flex-1 items-center gap-1 text-left"
          aria-expanded={!collapsed}
        >
          <ChevronDown
            size={11}
            className={`text-ink-4 transition-transform duration-150 ${collapsed ? '-rotate-90' : ''}`}
          />
          <span className="kicker group-hover:text-ink-2">{category.name}</span>
        </button>
        {onAddChannel ? (
          <button
            type="button"
            onClick={onAddChannel}
            aria-label={`Create channel in ${category.name}`}
            className="grid h-4 w-4 place-items-center rounded text-ink-4 opacity-0 transition hover:text-ink-2 group-hover:opacity-100"
          >
            <Plus size={13} />
          </button>
        ) : null}
      </div>
      <div className="flex flex-col gap-px pl-1">
        {visible.map((c) => (
          <ChannelItem key={c.id} channel={c} />
        ))}
      </div>
    </section>
  );
}
