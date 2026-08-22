import { iconMap } from '../lib/iconMap';
import { useUI } from '../store';
import { Popover } from './ui/Popover';

export function ContextMenu() {
  const menu = useUI((s) => s.menu);
  const close = useUI((s) => s.closeMenu);
  if (!menu) return null;

  return (
    <Popover anchor={menu.anchor} onClose={close} width={220}>
      <div
        role="menu"
        className="overflow-hidden rounded-lg border border-line-2/70 bg-surface/95 p-1 shadow-pop backdrop-blur-md"
      >
        {menu.items.map((item, i) => {
          if (item.separator) return <div key={i} className="my-1 h-px bg-line-2/60" />;
          const Icon = item.icon ? iconMap[item.icon] : null;
          return (
            <button
              key={i}
              role="menuitem"
              disabled={item.disabled}
              onClick={() => {
                item.onSelect?.();
                close();
              }}
              className={`flex w-full items-center gap-2.5 rounded-md px-2 py-1.5 text-left text-[13px] font-medium transition-colors ${
                item.danger
                  ? 'text-dnd hover:bg-dnd/15'
                  : 'text-ink-2 hover:bg-surface-hover hover:text-ink'
              } ${item.disabled ? 'cursor-default opacity-40' : ''}`}
            >
              {Icon && <Icon size={15} strokeWidth={1.9} className="shrink-0" />}
              <span className="flex-1">{item.label}</span>
              {item.shortcut && (
                <span className="font-mono text-3xs text-ink-4">{item.shortcut}</span>
              )}
            </button>
          );
        })}
      </div>
    </Popover>
  );
}
