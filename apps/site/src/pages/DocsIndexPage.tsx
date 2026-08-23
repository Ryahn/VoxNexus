import { Link } from 'react-router-dom';
import { docsNav } from '@/lib/nav';

export function DocsIndexPage() {
  return (
    <div className="mx-auto max-w-3xl">
      <h1 className="mb-2 font-sans text-2xl font-semibold text-ink">Documentation</h1>
      <p className="mb-8 text-ink-2">
        Setup, product behavior, and contributor notes for running and extending VoxNexus.
      </p>
      <div className="space-y-8">
        {docsNav.map((section) => (
          <section key={section.title}>
            <h2 className="kicker mb-3">{section.title}</h2>
            <ul className="space-y-2">
              {section.items.map((item) => (
                <li key={item.to}>
                  <Link
                    to={item.to}
                    className="block rounded-lg border border-line/60 bg-panel/40 px-4 py-3 text-sm text-ink transition-colors hover:border-line-2 hover:bg-surface/40"
                  >
                    {item.label}
                  </Link>
                </li>
              ))}
            </ul>
          </section>
        ))}
      </div>
    </div>
  );
}
