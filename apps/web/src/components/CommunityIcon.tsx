import type { Community } from '../types';
import { Tooltip } from './ui/Tooltip';

export function CommunityIcon({
  community,
  active,
  onClick,
}: {
  community: Community;
  active: boolean;
  onClick: () => void;
}) {
  const hasMention = (community.mentions ?? 0) > 0;
  const hasUnread = active ? false : (community.unread ?? 0) > 0 || hasMention;

  return (
    <div className="group relative flex items-center justify-center">
      {/* left selection / unread rail marker */}
      <span
        aria-hidden
        className={`absolute -left-3 w-1 rounded-r-full bg-accent transition-all duration-200 ease-swift ${
          active
            ? 'h-7 opacity-100'
            : hasUnread
              ? 'h-2 opacity-90 group-hover:h-4'
              : 'h-0 opacity-0'
        }`}
        style={
          active
            ? undefined
            : { background: hasMention ? 'rgb(var(--mention))' : 'rgb(var(--text-2))' }
        }
      />

      <Tooltip label={community.name} side="right">
        <button
          type="button"
          onClick={onClick}
          aria-label={community.name}
          aria-current={active}
          className={`relative grid h-11 w-11 place-items-center overflow-hidden font-sans text-[13px] font-bold tracking-tight text-app outline-none transition-[border-radius,transform,box-shadow] duration-200 ease-swift ${
            active ? 'rounded-[34%]' : 'rounded-[50%] group-hover:rounded-[34%]'
          }`}
          style={{
            background: `linear-gradient(150deg, rgb(${community.accent}) 0%, rgb(${community.accent} / 0.55) 100%)`,
            boxShadow: active
              ? `0 0 0 2px rgb(var(--bg-rail)), 0 0 0 3.5px rgb(${community.accent} / 0.8)`
              : undefined,
          }}
        >
          {community.tag}
          <span
            aria-hidden
            className="pointer-events-none absolute inset-0"
            style={{
              background:
                'linear-gradient(180deg, rgb(255 255 255 / 0.16), transparent 45%, rgb(0 0 0 / 0.28))',
            }}
          />
        </button>
      </Tooltip>

      {/* mention count pill */}
      {hasMention && (
        <span className="absolute -bottom-1 -right-1 z-10 grid min-w-[16px] place-items-center rounded-full border-2 border-rail bg-[rgb(var(--mention))] px-1 font-mono text-3xs font-bold text-app">
          {community.mentions}
        </span>
      )}
    </div>
  );
}
