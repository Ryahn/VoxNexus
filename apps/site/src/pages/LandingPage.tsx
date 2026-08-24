import { ArrowRight, BookOpen, Server, Shield } from 'lucide-react';
import { Link } from 'react-router-dom';

export function LandingPage() {
  return (
    <div className="relative flex min-h-full flex-col">
      <div className="pointer-events-none absolute inset-0 grid-veil opacity-30" />
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_top,rgb(var(--accent)/0.08),transparent_60%)]" />

      <header className="relative z-10 flex h-14 items-center justify-between border-b border-line/60 px-6">
        <div className="flex items-center gap-2 font-sans text-sm font-semibold text-ink">
          <span className="inline-flex h-7 w-7 items-center justify-center rounded-md bg-accent/15 text-accent">
            V
          </span>
          VoxNexus
        </div>
        <nav className="flex items-center gap-4 text-sm">
          <Link to="/docs" className="text-ink-2 transition-colors hover:text-ink">
            Docs
          </Link>
          <Link
            to="/docs/api"
            className="rounded-md bg-accent/15 px-3 py-1.5 font-medium text-accent transition-colors hover:bg-accent/25"
          >
            API reference
          </Link>
        </nav>
      </header>

      <main className="relative z-10 mx-auto flex max-w-3xl flex-1 flex-col justify-center px-6 py-16">
        <p className="kicker mb-4 text-accent">Self-hostable community chat</p>
        <h1 className="mb-4 font-sans text-4xl font-semibold tracking-tight text-ink text-balance">
          Discord-class chat and voice, Guilded-class Spaces, built to self-host.
        </h1>
        <p className="mb-8 max-w-2xl text-lg leading-relaxed text-ink-2">
          VoxNexus is a Rust + React stack for private community instances — auth, profiles,
          communities, Spaces, channels, roles, permissions, and gateway realtime.
        </p>
        <div className="flex flex-wrap gap-3">
          <Link
            to="/docs/setup/compose"
            className="inline-flex items-center gap-2 rounded-lg bg-accent px-4 py-2.5 text-sm font-medium text-app transition-opacity hover:opacity-90"
          >
            Get started
            <ArrowRight className="h-4 w-4" />
          </Link>
          <Link
            to="/docs"
            className="inline-flex items-center gap-2 rounded-lg border border-line-2 bg-surface/60 px-4 py-2.5 text-sm font-medium text-ink transition-colors hover:bg-surface"
          >
            <BookOpen className="h-4 w-4" />
            Read the docs
          </Link>
        </div>

        <div className="mt-16 grid gap-4 sm:grid-cols-3">
          <FeatureCard
            icon={Server}
            title="One Compose stack"
            description="Postgres, Redis, SeaweedFS, Typesense, and the app on a single command."
          />
          <FeatureCard
            icon={Shield}
            title="Session auth"
            description="Local email/password sessions with cookie-based API access and profile management."
          />
          <FeatureCard
            icon={BookOpen}
            title="OpenAPI + gateway"
            description="Generated HTTP client, Scalar API reference, and WebSocket gateway protocol docs."
          />
        </div>
      </main>
    </div>
  );
}

function FeatureCard({
  icon: Icon,
  title,
  description,
}: {
  icon: typeof Server;
  title: string;
  description: string;
}) {
  return (
    <div className="rounded-xl border border-line/80 bg-panel/60 p-4 shadow-inset-line">
      <Icon className="mb-2 h-5 w-5 text-accent" />
      <h2 className="mb-1 font-sans text-sm font-semibold text-ink">{title}</h2>
      <p className="text-sm leading-relaxed text-ink-3">{description}</p>
    </div>
  );
}
