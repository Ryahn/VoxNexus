import { Bell, BellOff, GitBranch, Plus, Send, X } from 'lucide-react';
import { useState } from 'react';
import { nameColor } from '../data/roles';
import { threadReplies, threadRoot } from '../data/threads';
import { users } from '../data/users';
import { useUI } from '../store';
import { RichText } from './RichText';
import { Avatar } from './ui/Avatar';

export function ThreadPanel() {
  const setThreadOpen = useUI((s) => s.setThreadOpen);
  const [following, setFollowing] = useState(true);
  const [draft, setDraft] = useState('');
  const rootAuthor = users[threadRoot.authorId];
  const participants = Array.from(
    new Set([threadRoot.authorId, ...threadReplies.map((r) => r.authorId)]),
  );

  return (
    <aside className="flex h-full w-[300px] shrink-0 flex-col border-l border-line/70 bg-panel animate-slide-in-right xl:w-[360px]">
      {/* header */}
      <div className="flex h-12 shrink-0 items-center gap-2 border-b border-line/70 px-3">
        <GitBranch size={16} className="text-accent" />
        <div className="min-w-0 flex-1 leading-tight">
          <div className="truncate text-[13px] font-semibold text-ink">Thread</div>
          <div className="truncate font-mono text-3xs text-ink-3">
            Scroll jump when thread panel opens
          </div>
        </div>
        <button
          type="button"
          onClick={() => setFollowing((f) => !f)}
          aria-label={following ? 'Unfollow thread' : 'Follow thread'}
          className={`grid h-8 w-8 place-items-center rounded-md transition-colors ${
            following
              ? 'text-accent hover:bg-surface-hover'
              : 'text-ink-3 hover:bg-surface-hover hover:text-ink'
          }`}
        >
          {following ? <Bell size={16} /> : <BellOff size={16} />}
        </button>
        <button
          type="button"
          onClick={() => setThreadOpen(false)}
          aria-label="Close thread"
          className="grid h-8 w-8 place-items-center rounded-md text-ink-3 transition-colors hover:bg-surface-hover hover:text-ink"
        >
          <X size={17} />
        </button>
      </div>

      {/* participants strip */}
      <div className="flex items-center gap-2 border-b border-line/60 px-3 py-2">
        <div className="flex -space-x-2">
          {participants.map((id) =>
            users[id] ? (
              <Avatar
                key={id}
                user={users[id]}
                size={22}
                rounded="rounded-full"
                className="ring-2 ring-panel"
              />
            ) : null,
          )}
        </div>
        <span className="font-mono text-3xs text-ink-3">
          {participants.length} participants · {threadReplies.length} replies
        </span>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto px-3 py-3">
        {/* root */}
        <div className="rounded-lg border border-line-2/50 bg-surface/40 p-3">
          <div className="flex items-center gap-2">
            {rootAuthor && <Avatar user={rootAuthor} size={22} rounded="rounded-full" />}
            <span
              className="text-[13px] font-semibold"
              style={{ color: `rgb(${nameColor(rootAuthor?.roleIds ?? [])})` }}
            >
              {rootAuthor?.displayName}
            </span>
            <time className="font-mono text-3xs text-ink-4">{threadRoot.ts}</time>
          </div>
          <p className="mt-1.5 font-body text-[13.5px] leading-relaxed text-ink-2">
            {threadRoot.content}
          </p>
        </div>

        <div className="my-2 flex items-center gap-2">
          <div className="h-px flex-1 bg-line/70" />
          <span className="font-mono text-3xs uppercase tracking-wider text-ink-4">
            {threadReplies.length} replies
          </span>
          <div className="h-px flex-1 bg-line/70" />
        </div>

        {/* replies */}
        <div className="flex flex-col gap-3">
          {threadReplies.map((r) => {
            const a = users[r.authorId];
            if (!a) return null;
            return (
              <div key={r.id} className="flex gap-2.5">
                <Avatar user={a} size={30} rounded="rounded-[32%]" />
                <div className="min-w-0 flex-1">
                  <div className="flex items-baseline gap-2">
                    <span
                      className="text-[13px] font-semibold"
                      style={{ color: `rgb(${nameColor(a.roleIds)})` }}
                    >
                      {a.displayName}
                    </span>
                    <time className="font-mono text-[10px] text-ink-4">{r.ts}</time>
                  </div>
                  <p className="font-body text-[13.5px] leading-relaxed text-ink">
                    <RichText text={r.content} />
                  </p>
                  {r.reactions && (
                    <div className="mt-1 flex gap-1">
                      {r.reactions.map((re, i) => (
                        <span
                          key={i}
                          className={`flex items-center gap-1 rounded-md border px-1.5 py-0.5 text-xs ${
                            re.me
                              ? 'border-accent/50 bg-accent/12'
                              : 'border-line-2/50 bg-surface/60'
                          }`}
                        >
                          <span className="text-[13px]">{re.emoji}</span>
                          <span className="font-mono text-[11px] text-ink-2">{re.count}</span>
                        </span>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* mini composer */}
      <div className="shrink-0 px-3 pb-3">
        <div className="flex items-end gap-2 rounded-lg border border-line-2/60 bg-input px-2 py-1.5 focus-within:border-accent/60">
          <button className="mb-0.5 grid h-7 w-7 place-items-center rounded-md text-ink-2 hover:text-accent">
            <Plus size={18} />
          </button>
          <textarea
            rows={1}
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            placeholder="Reply to thread…"
            className="my-1 max-h-28 min-h-[22px] flex-1 resize-none bg-transparent font-body text-[13.5px] text-ink outline-none placeholder:text-ink-4"
          />
          <button
            aria-label="Send reply"
            disabled={!draft.trim()}
            className={`mb-0.5 grid h-7 w-7 place-items-center rounded-md ${
              draft.trim() ? 'bg-accent/90 text-app hover:bg-accent' : 'bg-surface/60 text-ink-4'
            }`}
          >
            <Send size={14} />
          </button>
        </div>
      </div>
    </aside>
  );
}
