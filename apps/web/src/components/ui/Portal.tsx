import { type ReactNode, useEffect, useState } from 'react';
import { createPortal } from 'react-dom';

/** Renders children into a document-level layer for overlays. */
export function Portal({ children }: { children: ReactNode }) {
  const [el] = useState(() => {
    const node = document.createElement('div');
    node.setAttribute('data-vox-portal', '');
    return node;
  });
  useEffect(() => {
    document.body.appendChild(el);
    return () => {
      document.body.removeChild(el);
    };
  }, [el]);
  return createPortal(children, el);
}
