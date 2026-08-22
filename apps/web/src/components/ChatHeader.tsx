import {
  Bell,
  Hash,
  Inbox,
  MessagesSquare,
  PanelLeft,
  Pin,
  Rows3,
  Search,
  Users,
} from 'lucide-react';
import { channels } from '../data/structure';
import { channelMeta } from '../lib/channelMeta';
import { useUI } from '../store';
import { IconButton } from './ui/IconButton';

export function ChatHeader() {
  const activeChannel = useUI((s) => s.activeChannel);
  const toggleMembers = useUI((s) => s.toggleMembers);
  const membersOpen = useUI((s) => s.membersOpen);
  const toggleNav = useUI((s) => s.toggleNav);
  const setSearchOpen = useUI((s) => s.setSearchOpen);
  const setThreadOpen = useUI((s) => s.setThreadOpen);
  const threadOpen = useUI((s) => s.threadOpen);
  const compact = useUI((s) => s.compact);
  const toggleCompact = useUI((s) => s.toggleCompact);

  const ch = channels.find((c) => c.id === activeChannel) ?? channels[0];
  const Icon = channelMeta[ch.type].Icon;

  return (
    <header className="relative z-10 flex h-12 shrink-0 items-center gap-2 border-b border-line/70 bg-app px-3">
      <IconButton icon={PanelLeft} label="Toggle sidebar" tipSide="bottom" onClick={toggleNav} />
      <div className="mx-1 h-5 w-px bg-line-2/60" />

      <div className="flex min-w-0 flex-1 items-center gap-2">
        <Icon size={18} className="shrink-0 text-ink-3" strokeWidth={2} />
        <h1 className="min-w-0 truncate font-sans text-[15px] font-semibold text-ink">{ch.name}</h1>
        {ch.topic && (
          <>
            <span className="mx-1 hidden h-4 w-px bg-line-2/70 lg:block" />
            <p className="hidden min-w-0 truncate text-[12.5px] text-ink-3 lg:block">{ch.topic}</p>
          </>
        )}
      </div>

      <div className="flex shrink-0 items-center gap-0.5">
        <span className="hidden xl:flex">
          <IconButton icon={Hash} label="Following" />
        </span>
        <span className="hidden md:flex">
          <IconButton
            icon={MessagesSquare}
            label="Threads"
            active={threadOpen}
            onClick={() => setThreadOpen(!threadOpen)}
          />
        </span>
        <span className="hidden lg:flex">
          <IconButton icon={Pin} label="Pinned Messages" />
        </span>
        <span className="hidden xl:flex">
          <IconButton
            icon={Rows3}
            label={compact ? 'Cozy layout' : 'Compact layout'}
            active={compact}
            onClick={toggleCompact}
          />
        </span>
        <IconButton icon={Users} label="Member List" active={membersOpen} onClick={toggleMembers} />
        <div className="mx-1 hidden h-5 w-px bg-line-2/60 md:block" />
        <span className="hidden md:flex">
          <IconButton
            icon={Search}
            label="Search"
            kbd="Ctrl K"
            onClick={() => setSearchOpen(true)}
          />
        </span>
        <span className="hidden lg:flex">
          <IconButton icon={Inbox} label="Inbox" />
        </span>
        <IconButton icon={Bell} label="Notification Settings" />
      </div>
    </header>
  );
}
