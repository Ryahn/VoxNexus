import { Fragment, type ReactNode } from 'react';

/* Minimal inline renderer for mock content:
   - @mentions  → accent pill
   - `code`      → inline code
   - **bold**    → bold
   Real markdown/AST parsing belongs in a later layer. */
export function RichText({ text }: { text: string }) {
  return <>{parse(text)}</>;
}

function parse(text: string): ReactNode {
  // split on mentions / inline code / bold, keeping delimiters
  const parts = text.split(/(@[\w.]+|`[^`]+`|\*\*[^*]+\*\*)/g);
  return parts.map((p, i) => {
    if (!p) return null;
    if (p.startsWith('@')) {
      return (
        <span
          key={i}
          className="cursor-pointer rounded bg-accent-2/15 px-1 font-medium text-accent-2 transition-colors hover:bg-accent-2/25"
        >
          {p}
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
