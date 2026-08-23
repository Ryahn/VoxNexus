import { BookOpen, ExternalLink, Github, Home } from 'lucide-react';
import type { ReactNode } from 'react';
import { NavLink } from 'react-router-dom';
import { docsNav } from '@/lib/nav';

const GITHUB_URL = 'https://github.com/voxnexus/voxnexus';

type DocsShellProps = {
  children: ReactNode;
  fullWidth?: boolean;
};

function TopBar() {
  const linkClass = ({ isActive }: { isActive: boolean }) =>
    `flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-sm transition-colors ${
      isActive ? 'bg-surface text-ink' : 'text-ink-2 hover:bg-surface/60 hover:text-ink'
    }`;

  return (
    <header className="flex h-12 shrink-0 items-center justify-between border-b border-line bg-rail/90 px-4 backdrop-blur-sm">
      <div className="flex items-center gap-6">
        <NavLink
          to="/"
          className="flex items-center gap-2 font-sans text-sm font-semibold text-ink"
        >
          <span className="inline-flex h-6 w-6 items-center justify-center rounded-md bg-accent/15 text-accent">
            V
          </span>
          VoxNexus
        </NavLink>
        <nav className="flex items-center gap-1">
          <NavLink to="/" className={linkClass} end>
            <Home className="h-3.5 w-3.5" />
            Home
          </NavLink>
          <NavLink to="/docs" className={linkClass}>
            <BookOpen className="h-3.5 w-3.5" />
            Docs
          </NavLink>
          <NavLink to="/docs/api" className={linkClass}>
            API
          </NavLink>
        </nav>
      </div>
      <a
        href={GITHUB_URL}
        target="_blank"
        rel="noopener noreferrer"
        className="flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-sm text-ink-2 transition-colors hover:bg-surface/60 hover:text-ink"
      >
        <Github className="h-3.5 w-3.5" />
        GitHub
        <ExternalLink className="h-3 w-3 opacity-50" />
      </a>
    </header>
  );
}

function Sidebar() {
  const itemClass = ({ isActive }: { isActive: boolean }) =>
    `block rounded-md px-2.5 py-1.5 text-sm transition-colors ${
      isActive ? 'bg-surface-active text-ink' : 'text-ink-2 hover:bg-surface/60 hover:text-ink'
    }`;

  return (
    <aside className="w-56 shrink-0 overflow-y-auto border-r border-line bg-panel/50 p-4">
      {docsNav.map((section) => (
        <div key={section.title} className="mb-5">
          <p className="kicker mb-2 px-2.5">{section.title}</p>
          <ul className="space-y-0.5">
            {section.items.map((item) => (
              <li key={item.to}>
                <NavLink to={item.to} className={itemClass} end={item.to === '/docs'}>
                  {item.label}
                </NavLink>
              </li>
            ))}
          </ul>
        </div>
      ))}
    </aside>
  );
}

export function DocsShell({ children, fullWidth = false }: DocsShellProps) {
  return (
    <div className="flex h-full min-h-0 flex-col">
      <TopBar />
      <div className="flex min-h-0 flex-1">
        {!fullWidth && <Sidebar />}
        <main className={`min-h-0 flex-1 overflow-y-auto ${fullWidth ? '' : 'p-6'}`}>
          {children}
        </main>
      </div>
    </div>
  );
}
