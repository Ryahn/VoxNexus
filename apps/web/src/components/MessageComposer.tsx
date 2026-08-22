import { AtSign, Bold, Gift, Plus, Send, Smile, Sticker, X } from 'lucide-react';
import { useRef, useState } from 'react';
import { messages } from '../data/messages';
import { channels } from '../data/structure';
import { users } from '../data/users';
import { useUI } from '../store';
import { Tooltip } from './ui/Tooltip';

export function MessageComposer({ placeholder }: { placeholder?: string } = {}) {
  const activeChannel = useUI((s) => s.activeChannel);
  const replyingTo = useUI((s) => s.replyingTo);
  const setReplyingTo = useUI((s) => s.setReplyingTo);
  const [value, setValue] = useState('');
  const ref = useRef<HTMLTextAreaElement>(null);

  const ch = channels.find((c) => c.id === activeChannel) ?? channels[0];
  const replyMsg = replyingTo ? messages.find((m) => m.id === replyingTo) : null;
  const replyAuthor = replyMsg ? users[replyMsg.authorId] : null;

  const grow = () => {
    const el = ref.current;
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = Math.min(el.scrollHeight, 220) + 'px';
  };

  const send = () => {
    setValue('');
    setReplyingTo(null);
    if (ref.current) ref.current.style.height = 'auto';
  };

  return (
    <div className="shrink-0 px-4 pb-4 pt-1">
      {/* reply bar */}
      {replyMsg && replyAuthor && (
        <div className="flex items-center gap-2 rounded-t-lg border border-b-0 border-line-2/60 bg-panel-2 px-3 py-1.5 text-[12px]">
          <span className="text-ink-3">Replying to</span>
          <span className="font-semibold" style={{ color: `rgb(${replyAuthor.accent})` }}>
            {replyAuthor.displayName}
          </span>
          <span className="truncate text-ink-3">{replyMsg.content}</span>
          <button
            type="button"
            aria-label="Cancel reply"
            onClick={() => setReplyingTo(null)}
            className="ml-auto grid h-5 w-5 place-items-center rounded text-ink-3 hover:bg-surface-hover hover:text-ink"
          >
            <X size={13} />
          </button>
        </div>
      )}

      <div
        className={`group flex items-end gap-2 border border-line-2/60 bg-input px-2 py-1.5 transition-colors focus-within:border-accent/60 focus-within:shadow-accent-glow ${
          replyMsg ? 'rounded-b-lg rounded-t-none' : 'rounded-lg'
        }`}
      >
        <Tooltip label="Attach file" side="top">
          <button
            type="button"
            aria-label="Attach"
            className="mb-0.5 grid h-8 w-8 shrink-0 place-items-center rounded-md text-ink-2 transition-colors hover:bg-surface-hover hover:text-accent"
          >
            <Plus size={20} strokeWidth={2} />
          </button>
        </Tooltip>

        <textarea
          ref={ref}
          rows={1}
          value={value}
          onChange={(e) => {
            setValue(e.target.value);
            grow();
          }}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault();
              if (value.trim()) send();
            }
          }}
          placeholder={placeholder ?? `Message #${ch.name}`}
          className="my-1 max-h-[220px] min-h-[24px] flex-1 resize-none bg-transparent font-body text-[14px] leading-relaxed text-ink outline-none placeholder:text-ink-4"
        />

        <div className="mb-0.5 flex shrink-0 items-center gap-0.5">
          <span className="hidden xl:flex">
            <CBtn icon={AtSign} label="Mention" />
          </span>
          <span className="hidden lg:flex">
            <CBtn icon={Bold} label="Formatting" />
          </span>
          <CBtn icon={Gift} label="GIF" />
          <span className="hidden sm:flex">
            <CBtn icon={Sticker} label="Sticker" />
          </span>
          <CBtn icon={Smile} label="Emoji" accent />
          <button
            type="button"
            aria-label="Send message"
            disabled={!value.trim()}
            onClick={() => value.trim() && send()}
            className={`ml-1 grid h-8 w-8 place-items-center rounded-md transition-all ${
              value.trim()
                ? 'bg-accent/90 text-app hover:bg-accent'
                : 'cursor-default bg-surface/60 text-ink-4'
            }`}
          >
            <Send size={16} strokeWidth={2} />
          </button>
        </div>
      </div>

      <div className="mt-1 flex flex-wrap items-center gap-x-3 gap-y-0.5 px-1 font-mono text-3xs text-ink-4">
        <span>
          <span className="text-ink-3">Enter</span> to send
        </span>
        <span>
          <span className="text-ink-3">Shift + Enter</span> for newline
        </span>
        <span className="ml-auto">Markdown supported</span>
      </div>
    </div>
  );
}

function CBtn({
  icon: Icon,
  label,
  accent,
}: {
  icon: typeof Smile;
  label: string;
  accent?: boolean;
}) {
  return (
    <Tooltip label={label} side="top">
      <button
        type="button"
        aria-label={label}
        className={`grid h-8 w-8 place-items-center rounded-md text-ink-2 transition-colors hover:bg-surface-hover ${
          accent ? 'hover:text-[rgb(var(--mention))]' : 'hover:text-ink'
        }`}
      >
        <Icon size={18} strokeWidth={1.9} />
      </button>
    </Tooltip>
  );
}
