import { Check, Copy, FileText, GitBranch, Pin, Reply } from 'lucide-react';
import { nameColor, topRole } from '../data/roles';
import { users } from '../data/users';
import { useUI } from '../store';
import type { Message as Msg, Reaction, User } from '../types';
import { MessageActions } from './MessageActions';
import { RichText } from './RichText';
import { Avatar } from './ui/Avatar';

export function Message({
  message,
  grouped,
  author: authorProp,
  canEdit,
  canDelete,
  onEdit,
  onDelete,
  onJumpToReply,
  mentionLabels,
}: {
  message: Msg;
  grouped: boolean;
  author?: User;
  canEdit?: boolean;
  canDelete?: boolean;
  onEdit?: () => void;
  onDelete?: () => void;
  onJumpToReply?: (messageId: string) => void;
  mentionLabels?: Record<string, string>;
}) {
  const author = authorProp ?? users[message.authorId];
  const compact = useUI((s) => s.compact);
  const openProfile = useUI((s) => s.openProfile);
  const setThreadOpen = useUI((s) => s.setThreadOpen);
  if (!author) return null;

  const color = nameColor(author.roleIds);
  const role = topRole(author.roleIds);

  const openAuthor = (e: React.MouseEvent) => {
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    openProfile(author.id, { x: r.right + 8, y: r.top });
  };

  return (
    <div
      id={`msg-${message.id}`}
      className={`group/msg relative ${compact ? 'py-px' : grouped ? 'py-0.5' : 'mt-3.5 py-0.5'} ${
        message.mentionsMe ? 'bg-[rgb(var(--mention)/0.07)]' : 'hover:bg-surface/40'
      } transition-colors`}
    >
      {message.mentionsMe && (
        <span className="absolute inset-y-0 left-0 w-0.5 bg-[rgb(var(--mention))]" />
      )}
      {message.pinned && !grouped && (
        <div className="mb-0.5 flex items-center gap-1.5 pl-[68px] font-mono text-3xs uppercase tracking-wider text-accent/80">
          <Pin size={10} /> Pinned by Nova
        </div>
      )}

      <MessageActions
        message={message}
        canEdit={canEdit}
        canDelete={canDelete}
        onEdit={onEdit}
        onDelete={onDelete}
      />

      <div className={`flex gap-3 ${compact ? 'px-4' : 'px-4'}`}>
        {/* gutter: avatar or hover-timestamp */}
        <div className={`${compact ? 'w-0' : 'w-10'} shrink-0`}>
          {!grouped && !compact && (
            <button type="button" onClick={openAuthor} className="mt-0.5 block">
              <Avatar user={author} size={40} rounded="rounded-[32%]" />
            </button>
          )}
          {(grouped || compact) && (
            <span className="mt-1 hidden select-none justify-end pr-1 font-mono text-[10px] leading-5 text-ink-4 group-hover/msg:flex">
              {message.ts}
            </span>
          )}
        </div>

        <div className="min-w-0 flex-1">
          {/* reply context */}
          {message.replyTo && <ReplyContext reply={message.replyTo} onJump={onJumpToReply} />}

          {/* header line */}
          {(!grouped || compact) && (
            <div className={`flex items-baseline gap-2 ${compact ? 'inline-flex mr-2' : ''}`}>
              <button
                type="button"
                onClick={openAuthor}
                className="font-sans text-[14px] font-semibold hover:underline"
                style={{ color: `rgb(${color})` }}
              >
                {author.displayName}
              </button>
              {role && role.rank >= 70 && (
                <span
                  className="rounded px-1 py-px font-mono text-[9px] font-semibold uppercase tracking-wider"
                  style={{ color: `rgb(${role.color})`, background: `rgb(${role.color} / 0.14)` }}
                >
                  {role.name}
                </span>
              )}
              {!compact && (
                <time className="font-mono text-[10.5px] text-ink-4" title={message.fullTs}>
                  {message.fullTs ?? message.ts}
                </time>
              )}
            </div>
          )}

          {/* body */}
          {message.content && (
            <div
              className={`font-body text-[14px] leading-[1.5] text-ink-2 ${compact ? 'inline' : ''}`}
            >
              <span className="text-ink">
                <RichText text={message.content} labels={mentionLabels} />
              </span>
              {message.edited && (
                <span className="ml-1 font-mono text-[10px] text-ink-4">(edited)</span>
              )}
            </div>
          )}

          {message.code && <CodeBlock lang={message.code.lang} body={message.code.body} />}
          {message.attachments?.map((a) => (
            <Attachment key={a.id} att={a} />
          ))}
          {message.embeds?.map((e, i) => (
            <LinkEmbed key={i} embed={e} />
          ))}

          {message.reactions && <Reactions reactions={message.reactions} />}

          {message.thread && (
            <button
              type="button"
              onClick={() => setThreadOpen(true)}
              className="group/th mt-1.5 flex w-full min-w-0 max-w-[560px] items-center gap-2 rounded-lg border border-line-2/50 bg-surface/50 py-1 pl-2 pr-3 text-left transition-colors hover:border-accent/40 hover:bg-surface"
            >
              <GitBranch size={14} className="shrink-0 text-accent" />
              <span className="flex shrink-0 -space-x-1.5">
                {message.thread.participantIds
                  .slice(0, 3)
                  .map((id) =>
                    users[id] ? (
                      <Avatar
                        key={id}
                        user={users[id]}
                        size={18}
                        rounded="rounded-full"
                        className="ring-2 ring-app"
                      />
                    ) : null,
                  )}
              </span>
              <span className="shrink-0 text-[12.5px] font-semibold text-accent">
                {message.thread.replyCount} replies
              </span>
              <span className="min-w-0 truncate text-[12px] text-ink-3">
                {message.thread.title}
              </span>
              <span className="ml-auto shrink-0 font-mono text-3xs text-ink-4">
                {message.thread.lastReplyAt}
              </span>
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function ReplyContext({
  reply,
  onJump,
}: {
  reply: NonNullable<Msg['replyTo']>;
  onJump?: (messageId: string) => void;
}) {
  const author = users[reply.authorId];
  const displayName =
    reply.authorDisplayName ??
    author?.displayName ??
    (reply.deleted ? 'Deleted message' : 'Unknown');
  return (
    <button
      type="button"
      onClick={() => onJump?.(reply.messageId)}
      className="group/r mb-0.5 flex w-full min-w-0 items-center gap-1.5 pl-1 text-left"
    >
      <span className="ml-[-4px] h-2.5 w-4 shrink-0 rounded-tl-md border-l-2 border-t-2 border-line-2/70" />
      {author && <Avatar user={author} size={16} rounded="rounded-full" />}
      <Reply size={11} className="shrink-0 text-ink-4" />
      <span className="shrink-0 text-[12px] font-semibold text-ink-2 group-hover/r:text-ink">
        {displayName}
      </span>
      <span className="truncate text-[12px] text-ink-3 group-hover/r:text-ink-2">
        {reply.deleted ? 'Original message was deleted' : reply.excerpt}
      </span>
    </button>
  );
}

function CodeBlock({ lang, body }: { lang: string; body: string }) {
  return (
    <div className="my-1.5 w-full max-w-[640px] overflow-hidden rounded-lg border border-line-2/60 bg-[rgb(var(--input))]">
      <div className="flex items-center justify-between border-b border-line-2/50 bg-surface/60 px-3 py-1">
        <span className="flex items-center gap-1.5 font-mono text-3xs uppercase tracking-wider text-ink-3">
          <span className="h-1.5 w-1.5 rounded-full bg-accent/70" />
          {lang}
        </span>
        <button className="flex items-center gap-1 font-mono text-3xs text-ink-3 transition-colors hover:text-ink">
          <Copy size={11} /> copy
        </button>
      </div>
      <pre className="overflow-x-auto px-3 py-2.5">
        <code className="font-mono text-[12.5px] leading-relaxed text-ink-2">{body}</code>
      </pre>
    </div>
  );
}

function Attachment({ att }: { att: NonNullable<Msg['attachments']>[number] }) {
  if (att.kind === 'image') {
    const src = att.thumbnailUrl ?? att.url;
    return (
      <div className="my-1.5 w-fit max-w-[440px] overflow-hidden rounded-lg border border-line-2/50">
        {src ? (
          <a href={att.url ?? src} target="_blank" rel="noreferrer">
            <img
              src={src}
              alt={att.name}
              className="max-h-[360px] max-w-full object-contain bg-surface/40"
            />
          </a>
        ) : (
          <div
            className="relative grid aspect-[2/1] place-items-center"
            style={{
              background: `linear-gradient(135deg, rgb(${att.hueA ?? '54 210 205'} / 0.35), rgb(${att.hueB ?? '138 124 246'} / 0.35))`,
            }}
          >
            <div className="absolute inset-0 grid-veil opacity-40" />
            <span className="z-10 rounded-md border border-white/15 bg-black/25 px-2.5 py-1 font-mono text-2xs text-ink/90 backdrop-blur-sm">
              {att.name}
            </span>
          </div>
        )}
        <div className="flex items-center justify-between bg-surface/60 px-2.5 py-1 font-mono text-3xs text-ink-3">
          <span>{att.name}</span>
          <span>{att.meta}</span>
        </div>
      </div>
    );
  }
  return (
    <div className="my-1.5 flex w-fit items-center gap-2.5 rounded-lg border border-line-2/50 bg-surface/60 px-3 py-2">
      <FileText size={20} className="text-accent" />
      <div className="leading-tight">
        {att.url ? (
          <a
            href={att.url}
            target="_blank"
            rel="noreferrer"
            className="text-[13px] font-medium text-accent hover:underline"
          >
            {att.name}
          </a>
        ) : (
          <div className="text-[13px] font-medium text-ink">{att.name}</div>
        )}
        <div className="font-mono text-3xs text-ink-3">{att.meta}</div>
      </div>
    </div>
  );
}

function LinkEmbed({ embed }: { embed: NonNullable<Msg['embeds']>[number] }) {
  return (
    <div
      className="my-1.5 max-w-[460px] rounded-r-lg rounded-l-sm border-l-[3px] bg-surface/50 py-2 pl-3 pr-3"
      style={{ borderColor: `rgb(${embed.accent ?? '54 210 205'})` }}
    >
      <div className="font-mono text-3xs uppercase tracking-wider text-ink-3">{embed.site}</div>
      <div className="mt-0.5 text-[13.5px] font-semibold text-accent hover:underline">
        {embed.title}
      </div>
      <div className="mt-0.5 text-[12.5px] leading-snug text-ink-2">{embed.description}</div>
    </div>
  );
}

function Reactions({ reactions }: { reactions: Reaction[] }) {
  return (
    <div className="mt-1 flex flex-wrap items-center gap-1">
      {reactions.map((r, i) => (
        <button
          key={i}
          type="button"
          className={`flex items-center gap-1 rounded-md border px-1.5 py-0.5 text-xs transition-colors ${
            r.me
              ? 'border-accent/50 bg-accent/12 text-ink'
              : 'border-line-2/50 bg-surface/60 text-ink-2 hover:border-line-2 hover:bg-surface-hover'
          }`}
        >
          <span className="text-[13px] leading-none">{r.emoji}</span>
          <span className="font-mono text-[11px] font-medium tabular-nums">{r.count}</span>
        </button>
      ))}
      <button
        type="button"
        aria-label="Add reaction"
        className="grid h-[22px] w-[22px] place-items-center rounded-md border border-line-2/40 text-ink-3 opacity-0 transition-all hover:border-line-2 hover:text-ink group-hover/msg:opacity-100"
      >
        <Check size={0} />
        <span className="text-[13px]">+</span>
      </button>
    </div>
  );
}
