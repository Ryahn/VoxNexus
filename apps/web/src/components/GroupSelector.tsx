import {
  ChevronRight,
  Code2,
  Home,
  Languages,
  Lock,
  type LucideIcon,
  Palette,
  Shield,
} from 'lucide-react';
import { groups } from '../data/structure';
import { menuFor } from '../lib/menus';
import { useUI } from '../store';

const groupIcons: Record<string, LucideIcon> = {
  home: Home,
  code: Code2,
  palette: Palette,
  languages: Languages,
  shield: Shield,
};

export function GroupSelector() {
  const active = useUI((s) => s.activeGroup);
  const setGroup = useUI((s) => s.setGroup);
  const collapsed = useUI((s) => s.collapsedSections);
  const toggleSection = useUI((s) => s.toggleSection);
  const openMenu = useUI((s) => s.openMenu);

  // preserve declared order of sections
  const sections: string[] = [];
  for (const g of groups) if (!sections.includes(g.section)) sections.push(g.section);

  return (
    <div className="border-b border-line/60 px-2 py-2">
      {sections.map((section) => {
        const isCollapsed = collapsed[section];
        const items = groups.filter((g) => g.section === section);
        return (
          <div key={section} className="mb-1.5 last:mb-0">
            <button
              type="button"
              onClick={() => toggleSection(section)}
              className="group flex w-full items-center gap-1 px-1.5 py-1 text-left"
            >
              <ChevronRight
                size={11}
                className={`text-ink-4 transition-transform duration-150 ${isCollapsed ? '' : 'rotate-90'}`}
              />
              <span className="kicker group-hover:text-ink-2">{section}</span>
            </button>
            {!isCollapsed && (
              <div className="mt-0.5 flex flex-col gap-0.5">
                {items.map((g) => {
                  const Icon = groupIcons[g.icon ?? 'home'] ?? Home;
                  const isActive = active === g.id;
                  return (
                    <button
                      key={g.id}
                      type="button"
                      onClick={() => setGroup(g.id)}
                      onContextMenu={(e) => {
                        e.preventDefault();
                        openMenu({ x: e.clientX, y: e.clientY }, menuFor('group', g.name));
                      }}
                      aria-current={isActive}
                      className={`group relative flex items-center gap-2 rounded-md px-2 py-1.5 text-left transition-colors duration-150 ${
                        isActive
                          ? 'bg-surface-active text-ink'
                          : 'text-ink-2 hover:bg-surface-hover/70 hover:text-ink'
                      }`}
                    >
                      {isActive && <span className="tick" />}
                      <Icon
                        size={15}
                        strokeWidth={1.9}
                        className={isActive ? 'text-accent' : 'text-ink-3 group-hover:text-ink-2'}
                      />
                      <span className="flex-1 truncate text-[13px] font-medium">{g.name}</span>
                      {g.restricted && <Lock size={11} className="text-ink-4" />}
                      {g.unread && !isActive && (
                        <span className="h-1.5 w-1.5 rounded-full bg-ink-2" />
                      )}
                    </button>
                  );
                })}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
