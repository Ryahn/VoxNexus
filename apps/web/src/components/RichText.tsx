/**
 * Lightweight message markup renderer (mentions, inline code, bold).
 * Mentions use `@{uuid}` / `@&{uuid}` / `@everyone` / `@here`.
 */
import { Fragment, type ReactNode } from 'react';

export function RichText({ text, labels }: { text: string; labels?: Record<string, string> }) {
  return <>{parse(text, labels)}</>;
}

function parse(text: string, labels?: Record<string, string>): ReactNode {
  const parts = text.split(
    /(@everyone|@here|@&\{[0-9a-fA-F-]{36}\}|@\{[0-9a-fA-F-]{36}\}|`[^`]+`|\*\*[^*]+\*\*)/g,
  );
  return parts.map((p, i) => {
    if (!p) return null;
    if (p === '@everyone' || p === '@here') {
      return (
        <span
          key={i}
          className="cursor-pointer rounded bg-accent-2/15 px-1 font-medium text-accent-2 transition-colors hover:bg-accent-2/25"
        >
          {p}
        </span>
      );
    }
    const role = p.match(/^@&\{([0-9a-fA-F-]{36})\}$/);
    if (role) {
      const id = role[1];
      const label = labels?.[id] ?? labels?.[`role:${id}`] ?? 'role';
      return (
        <span
          key={i}
          className="cursor-pointer rounded bg-accent-2/15 px-1 font-medium text-accent-2 transition-colors hover:bg-accent-2/25"
        >
          @{label}
        </span>
      );
    }
    const user = p.match(/^@\{([0-9a-fA-F-]{36})\}$/);
    if (user) {
      const id = user[1];
      const label = labels?.[id] ?? 'member';
      return (
        <span
          key={i}
          className="cursor-pointer rounded bg-accent-2/15 px-1 font-medium text-accent-2 transition-colors hover:bg-accent-2/25"
        >
          @{label}
        </span>
      );
    }
    if (p.startsWith('`') && p.endsWith('`')) {
      return (
        <code
          key={i}
          className="rounded border border-line-2/50 bg-app/80 px-1 py-px font-mono text-[12px] text-ink"
        >
          {p.slice(1, -1)}
        </code>
      );
    }
    if (p.startsWith('**') && p.endsWith('**')) {
      return (
        <strong key={i} className="font-semibold text-ink">
          {p.slice(2, -2)}
        </strong>
      );
    }
    return <Fragment key={i}>{p}</Fragment>;
  });
}
