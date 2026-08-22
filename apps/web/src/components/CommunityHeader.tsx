import { Bell, ChevronDown, Search, Sparkles } from 'lucide-react';
import { communities } from '../data/communities';
import { menuFor } from '../lib/menus';
import { useUI } from '../store';

export function CommunityHeader() {
  const activeId = useUI((s) => s.activeCommunity);
  const community = communities.find((c) => c.id === activeId) ?? communities[0];
  const openMenu = useUI((s) => s.openMenu);
  const setSearchOpen = useUI((s) => s.setSearchOpen);
  const setNotifOpen = useUI((s) => s.setNotifOpen);

  return (
    <header className="relative border-b border-line/70">
      {/* faint community-accent wash behind the header */}
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 opacity-[0.14]"
        style={{
          background: `radial-gradient(120% 140% at 0% 0%, rgb(${community.accent}), transparent 60%)`,
        }}
      />
      <button
        type="button"
        onClick={(e) => {
          const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
          openMenu({ x: r.left + 8, y: r.bottom + 6 }, menuFor('community', community.name));
        }}
        className="group relative flex w-full items-center gap-2.5 px-3 py-2.5 text-left transition-colors hover:bg-surface-hover/50"
      >
        <span
          className="grid h-7 w-7 shrink-0 place-items-center rounded-[30%] font-sans text-[11px] font-bold text-app"
          style={{
            background: `linear-gradient(150deg, rgb(${community.accent}), rgb(${community.accent} / 0.6))`,
          }}
        >
          {community.tag}
        </span>
        <span className="min-w-0 flex-1">
          <span className="flex items-center gap-1.5">
            <span className="truncate font-sans text-[13.5px] font-semibold text-ink">
              {community.name}
            </span>
            <Sparkles size={12} className="shrink-0 text-accent/80" />
          </span>
          <span className="block truncate font-mono text-3xs uppercase tracking-wider text-ink-3">
            11 online · 11 members
          </span>
        </span>
        <ChevronDown
          size={16}
          className="shrink-0 text-ink-3 transition-transform group-hover:text-ink-2"
        />
      </button>

      <div className="relative flex items-center gap-1 px-2 pb-2">
        <button
          type="button"
          onClick={() => setSearchOpen(true)}
          className="flex flex-1 items-center gap-2 rounded-md border border-line-2/50 bg-input px-2.5 py-1.5 text-ink-3 transition-colors hover:border-accent/40 hover:text-ink-2"
        >
          <Search size={14} />
          <span className="flex-1 text-left text-xs">Search</span>
          <kbd className="rounded border border-line-2/60 bg-app px-1 py-px font-mono text-3xs text-ink-3">
            Ctrl K
          </kbd>
        </button>
        <button
          type="button"
          onClick={() => setNotifOpen(true)}
          aria-label="Community notifications"
          className="grid h-8 w-8 shrink-0 place-items-center rounded-md text-ink-2 transition-colors hover:bg-surface-hover hover:text-ink"
        >
          <Bell size={15} strokeWidth={1.9} />
        </button>
      </div>
    </header>
  );
}
