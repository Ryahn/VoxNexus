/**
 * Lightweight message markup renderer (mentions, markdown subset, spoilers).
 * Never uses dangerouslySetInnerHTML — HTML in content stays plain text.
 */
import { Fragment, type ReactNode, useState } from 'react';
import { type BlockNode, type InlineNode, parseMessageMarkup } from '../lib/messageMarkup';

export function RichText({ text, labels }: { text: string; labels?: Record<string, string> }) {
  const blocks = parseMessageMarkup(text);
  return (
    <>
      {blocks.map((block, i) => (
        <Fragment key={i}>{renderBlock(block, labels)}</Fragment>
      ))}
    </>
  );
}

function renderBlock(block: BlockNode, labels?: Record<string, string>): ReactNode {
  if (block.type === 'codeblock') {
    return (
      <pre className="my-1.5 max-w-[640px] overflow-x-auto rounded-lg border border-line-2/60 bg-[rgb(var(--input))] p-3 font-mono text-[12.5px] leading-relaxed text-ink">
        {block.lang !== 'text' ? (
          <span className="mb-1 block font-mono text-3xs uppercase tracking-wider text-ink-4">
            {block.lang}
          </span>
        ) : null}
        <code>{block.text}</code>
      </pre>
    );
  }
  return (
    <span className="whitespace-pre-wrap">
      {block.children.map((child, i) => (
        <Fragment key={i}>{renderInline(child, labels)}</Fragment>
      ))}
    </span>
  );
}

function Spoiler({ children }: { children: ReactNode }) {
  const [revealed, setRevealed] = useState(false);
  return (
    <button
      type="button"
      onClick={() => setRevealed(true)}
      aria-label={revealed ? undefined : 'Spoiler — click to reveal'}
      className={
        revealed
          ? 'rounded bg-surface/80 px-0.5 text-left text-ink'
          : 'cursor-pointer rounded bg-ink/85 px-0.5 text-transparent transition-colors hover:bg-ink/70'
      }
    >
      <span className={revealed ? undefined : 'select-none'}>{children}</span>
    </button>
  );
}

function mentionClass() {
  return 'cursor-pointer rounded bg-accent-2/15 px-1 font-medium text-accent-2 transition-colors hover:bg-accent-2/25';
}

function renderInline(node: InlineNode, labels?: Record<string, string>): ReactNode {
  switch (node.type) {
    case 'text':
      return node.text;
    case 'everyone':
      return <span className={mentionClass()}>@everyone</span>;
    case 'here':
      return <span className={mentionClass()}>@here</span>;
    case 'user': {
      const label = labels?.[node.id] ?? 'member';
      return <span className={mentionClass()}>@{label}</span>;
    }
    case 'role': {
      const label = labels?.[node.id] ?? labels?.[`role:${node.id}`] ?? 'role';
      return <span className={mentionClass()}>@{label}</span>;
    }
    case 'code':
      return (
        <code className="rounded border border-line-2/50 bg-app/80 px-1 py-px font-mono text-[12px] text-ink">
          {node.text}
        </code>
      );
    case 'bold':
      return (
        <strong className="font-semibold text-ink">
          {node.children.map((c, i) => (
            <Fragment key={i}>{renderInline(c, labels)}</Fragment>
          ))}
        </strong>
      );
    case 'italic':
      return (
        <em>
          {node.children.map((c, i) => (
            <Fragment key={i}>{renderInline(c, labels)}</Fragment>
          ))}
        </em>
      );
    case 'strike':
      return (
        <s className="text-ink-2">
          {node.children.map((c, i) => (
            <Fragment key={i}>{renderInline(c, labels)}</Fragment>
          ))}
        </s>
      );
    case 'spoiler':
      return (
        <Spoiler>
          {node.children.map((c, i) => (
            <Fragment key={i}>{renderInline(c, labels)}</Fragment>
          ))}
        </Spoiler>
      );
    default:
      return null;
  }
}
