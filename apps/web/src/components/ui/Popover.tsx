import { type ReactNode, useEffect, useLayoutEffect, useRef, useState } from 'react';
import type { Anchor } from '../../store';
import { Portal } from './Portal';

/* Portal popover anchored to a point. Measures itself and clamps
   into the viewport. Closes on outside-click and Escape. */
export function Popover({
  anchor,
  onClose,
  children,
  className = '',
  width,
}: {
  anchor: Anchor;
  onClose: () => void;
  children: ReactNode;
  className?: string;
  width?: number;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState<{ left: number; top: number } | null>(null);

  // Runs after the Portal node is attached to the DOM (Portal appends
  // in its own effect), so the measurement is valid and the clamp works.
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const { width: w, height: h } = el.getBoundingClientRect();
    const m = 8;
    let left = anchor.left ? anchor.x - w : anchor.x;
    let top = anchor.bottom ? anchor.y - h : anchor.y;
    left = Math.max(m, Math.min(left, window.innerWidth - w - m));
    top = Math.max(m, Math.min(top, window.innerHeight - h - m));
    setPos({ left, top });
  }, [anchor]);

  useLayoutEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && onClose();
    const onDown = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    };
    document.addEventListener('keydown', onKey);
    // defer to avoid catching the opening click
    const t = setTimeout(() => document.addEventListener('mousedown', onDown), 0);
    return () => {
      document.removeEventListener('keydown', onKey);
      document.removeEventListener('mousedown', onDown);
      clearTimeout(t);
    };
  }, [onClose]);

  return (
    <Portal>
      <div
        ref={ref}
        className={`fixed z-[900] animate-pop-in ${className}`}
        style={{
          left: pos?.left ?? anchor.x,
          top: pos?.top ?? anchor.y,
          width,
          visibility: pos ? 'visible' : 'hidden',
        }}
      >
        {children}
      </div>
    </Portal>
  );
}
