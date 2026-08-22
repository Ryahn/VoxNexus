import { useEffect, useRef } from 'react';
import { messagesForChannel } from '../data/messages';
import { channels } from '../data/structure';
import { channelMeta } from '../lib/channelMeta';
import { useUI } from '../store';
import { Message } from './Message';

export function MessageList() {
  const activeChannel = useUI((s) => s.activeChannel);
  const list = messagesForChannel(activeChannel);
  const ch = channels.find((c) => c.id === activeChannel) ?? channels[0];
  const { Icon, label } = channelMeta[ch.type];
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView();
  }, [activeChannel]);

  return (
    <div className="min-h-0 flex-1 overflow-y-auto overflow-x-hidden overscroll-contain">
      {/* channel intro */}
      <div className="px-4 pb-2 pt-6">
        <div
          className="mb-3 grid h-14 w-14 place-items-center rounded-2xl border border-line-2/60"
          style={{
            background: 'linear-gradient(150deg, rgb(var(--surface)), rgb(var(--panel-2)))',
          }}
        >
          <Icon size={26} className="text-accent" strokeWidth={1.8} />
        </div>
        <h2 className="font-sans text-[22px] font-bold text-ink">
          Welcome to <span className="text-accent">#{ch.name}</span>
        </h2>
        <p className="mt-1 max-w-xl text-[13.5px] leading-relaxed text-ink-2">
          {ch.topic ?? `This is the start of the ${label.toLowerCase()} channel.`} This is the
          beginning of the channel’s history.
        </p>
        <div className="mt-3 flex items-center gap-2">
          <span className="chip">{label} channel</span>
          <span className="chip">Development ▸ General</span>
        </div>
      </div>

      {/* day divider */}
      <div className="flex items-center gap-3 px-4 py-1">
        <div className="h-px flex-1 bg-line/70" />
        <span className="rounded-full border border-line-2/50 bg-panel px-2.5 py-0.5 font-mono text-3xs uppercase tracking-wider text-ink-3">
          Today · August 22
        </span>
        <div className="h-px flex-1 bg-line/70" />
      </div>

      <div className="pb-3">
        {list.map((m, i) => {
          const prev = list[i - 1];
          const grouped = Boolean(
            prev &&
              prev.authorId === m.authorId &&
              !m.replyTo &&
              !m.pinned &&
              !prev.thread &&
              !prev.system,
          );
          // "new" divider before first mention of me for demo
          const showNew = m.id === 'm6';
          return (
            <div key={m.id}>
              {showNew && (
                <div className="flex items-center gap-2 px-4 py-1">
                  <div className="h-px flex-1 bg-[rgb(var(--mention)/0.5)]" />
                  <span className="font-mono text-3xs font-semibold uppercase tracking-wider text-[rgb(var(--mention))]">
                    New
                  </span>
                </div>
              )}
              <Message message={m} grouped={grouped} />
            </div>
          );
        })}
      </div>
      <div ref={bottomRef} />
    </div>
  );
}
