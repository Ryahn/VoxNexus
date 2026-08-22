import type { ReactNode } from 'react';

export type HelloPanelProps = {
  title: string;
  kicker: string;
  children: ReactNode;
};

export function HelloPanel({ title, kicker, children }: HelloPanelProps) {
  return (
    <section className="vn-hello-panel">
      <p className="vn-hello-kicker">{kicker}</p>
      <h1 className="vn-hello-title">{title}</h1>
      <div className="vn-hello-body">{children}</div>
    </section>
  );
}
