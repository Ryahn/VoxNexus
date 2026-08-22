import { avatarStyle } from '../../lib/avatar';
import type { User } from '../../types';
import { PresenceDot } from './Presence';

export function Avatar({
  user,
  size = 40,
  showPresence = false,
  ring = 'rgb(var(--bg-panel))',
  rounded = 'rounded-[30%]',
  className = '',
  dim = false,
}: {
  user: User;
  size?: number;
  showPresence?: boolean;
  ring?: string;
  rounded?: string;
  className?: string;
  dim?: boolean;
}) {
  const s = avatarStyle(user.avatarSeed, user.displayName);
  const dotSize = Math.max(9, Math.round(size * 0.3));
  return (
    <span
      className={`relative inline-block shrink-0 ${className}`}
      style={{ width: size, height: size }}
    >
      <span
        aria-hidden
        className={`grid h-full w-full place-items-center overflow-hidden ${rounded} font-sans font-semibold text-app`}
        style={{
          background: s.gradient,
          fontSize: size * 0.36,
          opacity: dim ? 0.45 : 1,
          filter: dim ? 'saturate(0.6)' : undefined,
        }}
      >
        {s.initials}
        {/* faint geometric overlay — cyberpunk identity detail */}
        <span
          className={`pointer-events-none absolute inset-0 ${rounded}`}
          style={{
            background:
              'radial-gradient(120% 120% at 15% 10%, rgb(255 255 255 / 0.18), transparent 42%), linear-gradient(180deg, transparent 60%, rgb(0 0 0 / 0.22))',
          }}
        />
      </span>
      {showPresence && (
        <span className="absolute -bottom-0.5 -right-0.5">
          <PresenceDot presence={user.presence} size={dotSize} ring={ring} />
        </span>
      )}
    </span>
  );
}
