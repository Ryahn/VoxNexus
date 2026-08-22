import { cloneElement, type ReactElement, type ReactNode, useRef, useState } from 'react';
import { Portal } from './Portal';

type Side = 'right' | 'top' | 'bottom' | 'left';

/* Hover/focus tooltip. Wraps a single interactive child, mirrors its
   ref-free bounding box, and renders a positioned label in a portal. */
export function Tooltip({
  label,
  side = 'right',
  children,
  kbd,
}: {
  label: ReactNode;
  side?: Side;
  children: ReactElement;
  kbd?: string;
}) {
  const ref = useRef<HTMLElement>(null);
  const [box, setBox] = useState<DOMRect | null>(null);

  const show = () => {
    const el = ref.current;
    if (el) setBox(el.getBoundingClientRect());
  };
  const hide = () => setBox(null);

  const child = cloneElement(children, {
    ref,
    onMouseEnter: show,
    onMouseLeave: hide,
    onFocus: show,
    onBlur: hide,
  } as never);

  return (
    <>
      {child}
      {box && (
        <Portal>
          <div
            role="tooltip"
            className="pointer-events-none fixed z-[999] animate-fade-in"
            style={pos(box, side)}
          >
            <div className="flex items-center gap-2 whitespace-nowrap rounded-md border border-line-2/70 bg-surface px-2.5 py-1.5 text-xs font-medium text-ink shadow-pop">
              {label}
              {kbd && (
                <kbd className="rounded border border-line-2/70 bg-app px-1 py-0.5 font-mono text-3xs text-ink-2">
                  {kbd}
                </kbd>
              )}
            </div>
          </div>
        </Portal>
      )}
    </>
  );
}

function pos(b: DOMRect, side: Side): React.CSSProperties {
  const gap = 10;
  switch (side) {
    case 'right':
      return { left: b.right + gap, top: b.top + b.height / 2, transform: 'translateY(-50%)' };
    case 'left':
      return { left: b.left - gap, top: b.top + b.height / 2, transform: 'translate(-100%,-50%)' };
    case 'top':
      return { left: b.left + b.width / 2, top: b.top - gap, transform: 'translate(-50%,-100%)' };
    case 'bottom':
      return { left: b.left + b.width / 2, top: b.bottom + gap, transform: 'translate(-50%,0)' };
  }
}
