import type { ReactNode } from 'react';

/**
 * Shared presentational primitives.
 *
 * The product UI shell and design tokens live in `apps/web` (VOX UI). Promote
 * reusable pieces here as Feature Tasks stabilize — do not rebuild a second
 * visual system in this package.
 */

export type HelloPanelProps = {
  title: string;
  kicker: string;
  children: ReactNode;
};

/** @deprecated Prefer shell surfaces in `apps/web`; kept for transitional stubs. */
export function HelloPanel({ title, kicker, children }: HelloPanelProps) {
  return (
    <section className="vn-hello-panel">
      <p className="vn-hello-kicker">{kicker}</p>
      <h1 className="vn-hello-title">{title}</h1>
      <div className="vn-hello-body">{children}</div>
    </section>
  );
}
