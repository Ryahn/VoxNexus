import type { Presence } from '../../types';

const meta: Record<Presence, { varName: string; label: string }> = {
  online: { varName: '--online', label: 'Online' },
  idle: { varName: '--idle', label: 'Idle' },
  dnd: { varName: '--dnd', label: 'Do Not Disturb' },
  offline: { varName: '--text-3', label: 'Offline' },
};

/* Presence indicator. Status is NOT color-only — each state carries a
   distinct inner cut-out (dot / crescent / bar / hollow) so it stays
   legible for color-blind users. `ring` is the surrounding surface
   color the badge is punched out of. */
export function PresenceDot({
  presence,
  size = 10,
  ring = 'rgb(var(--bg-panel))',
  className = '',
}: {
  presence: Presence;
  size?: number;
  ring?: string;
  className?: string;
}) {
  const { varName, label } = meta[presence];
  const color = `rgb(var(${varName}))`;
  return (
    <span
      role="img"
      aria-label={label}
      className={`relative inline-block shrink-0 rounded-full ${className}`}
      style={{ width: size, height: size, background: color, boxShadow: `0 0 0 2.5px ${ring}` }}
    >
      {presence === 'idle' && (
        <span
          className="absolute rounded-full"
          style={{
            width: size * 0.62,
            height: size * 0.62,
            top: 0,
            right: 0,
            transform: 'translate(6%,-6%)',
            background: ring,
          }}
        />
      )}
      {presence === 'dnd' && (
        <span
          className="absolute left-1/2 top-1/2 rounded-full"
          style={{
            width: size * 0.56,
            height: Math.max(2, size * 0.2),
            transform: 'translate(-50%,-50%)',
            background: ring,
          }}
        />
      )}
      {presence === 'offline' && (
        <span
          className="absolute left-1/2 top-1/2 rounded-full"
          style={{
            width: size * 0.5,
            height: size * 0.5,
            transform: 'translate(-50%,-50%)',
            background: ring,
          }}
        />
      )}
    </span>
  );
}
