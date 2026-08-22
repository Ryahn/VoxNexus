import type { LucideIcon } from 'lucide-react';
import { type ButtonHTMLAttributes, forwardRef } from 'react';
import { Tooltip } from './Tooltip';

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  icon: LucideIcon;
  label: string;
  tip?: boolean;
  tipSide?: 'top' | 'bottom' | 'left' | 'right';
  active?: boolean;
  size?: number;
  kbd?: string;
}

export const IconButton = forwardRef<HTMLButtonElement, Props>(function IconButton(
  {
    icon: Icon,
    label,
    tip = true,
    tipSide = 'bottom',
    active = false,
    size = 18,
    kbd,
    className = '',
    ...rest
  },
  ref,
) {
  const btn = (
    <button
      ref={ref}
      type="button"
      aria-label={label}
      aria-pressed={active}
      className={`grid h-8 w-8 place-items-center rounded-md text-ink-2 outline-none transition-colors duration-150 hover:bg-surface-hover hover:text-ink focus-visible:bg-surface-hover ${
        active ? 'bg-surface-active !text-accent' : ''
      } ${className}`}
      {...rest}
    >
      <Icon size={size} strokeWidth={1.9} />
    </button>
  );
  if (!tip) return btn;
  return (
    <Tooltip label={label} side={tipSide} kbd={kbd}>
      {btn}
    </Tooltip>
  );
});
