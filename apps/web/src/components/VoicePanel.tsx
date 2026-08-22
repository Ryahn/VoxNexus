import { Activity, MonitorUp, PhoneOff, Signal, Video, Volume2 } from 'lucide-react';
import { channels } from '../data/structure';
import { users } from '../data/users';
import { useUI } from '../store';
import { Avatar } from './ui/Avatar';
import { IconButton } from './ui/IconButton';

export function VoicePanel() {
  const voiceChannel = useUI((s) => s.voiceChannel);
  const disconnect = useUI((s) => s.disconnectVoice);
  const video = useUI((s) => s.video);
  const screenshare = useUI((s) => s.screenshare);
  const toggleVideo = useUI((s) => s.toggleVideo);
  const toggleScreenshare = useUI((s) => s.toggleScreenshare);

  const ch = channels.find((c) => c.id === voiceChannel);
  if (!ch) return null;
  const participants = ['me', ...(ch.connected ?? [])];

  return (
    <div className="relative border-t border-line/70 bg-panel-2 px-2.5 py-2">
      <div
        aria-hidden
        className="pointer-events-none absolute inset-x-0 top-0 h-px"
        style={{
          background: 'linear-gradient(90deg, transparent, rgb(var(--online) / 0.6), transparent)',
        }}
      />
      <div className="mb-1.5 flex items-center gap-2">
        <span className="relative grid h-6 w-6 place-items-center rounded-md bg-online/15 text-online">
          <Signal size={14} strokeWidth={2.2} />
        </span>
        <div className="min-w-0 flex-1 leading-tight">
          <div className="flex items-center gap-1.5">
            <span className="text-[11px] font-bold uppercase tracking-wider text-online">
              Voice Connected
            </span>
            <span className="h-1 w-1 animate-pulse rounded-full bg-online" />
          </div>
          <div className="flex items-center gap-1 truncate text-[12px] text-ink-2">
            <Volume2 size={12} className="shrink-0 text-ink-3" />
            <span className="truncate">{ch.name}</span>
            <span className="font-mono text-3xs text-ink-4">· 24ms</span>
          </div>
        </div>
        <IconButton
          icon={PhoneOff}
          label="Disconnect"
          tipSide="top"
          onClick={disconnect}
          className="!text-dnd hover:!bg-dnd/15"
        />
      </div>

      <div className="mb-1.5 flex items-center gap-1">
        {participants.map((uid) => {
          const u = users[uid];
          if (!u) return null;
          return (
            <div key={uid} className="ring-1 ring-online/40" style={{ borderRadius: 999 }}>
              <Avatar user={u} size={22} rounded="rounded-full" />
            </div>
          );
        })}
        <span className="ml-1 font-mono text-3xs text-ink-3">{participants.length} in call</span>
      </div>

      <div className="grid grid-cols-3 gap-1">
        <VoiceBtn
          icon={MonitorUp}
          label="Screen"
          active={screenshare}
          onClick={toggleScreenshare}
        />
        <VoiceBtn icon={Video} label="Camera" active={video} onClick={toggleVideo} />
        <VoiceBtn icon={Activity} label="Activity" />
      </div>
    </div>
  );
}

function VoiceBtn({
  icon: Icon,
  label,
  active,
  onClick,
}: {
  icon: typeof Signal;
  label: string;
  active?: boolean;
  onClick?: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`flex items-center justify-center gap-1.5 rounded-md border py-1.5 text-xs font-medium transition-colors ${
        active
          ? 'border-accent/40 bg-accent/12 text-accent'
          : 'border-line-2/50 bg-surface/60 text-ink-2 hover:bg-surface-hover hover:text-ink'
      }`}
    >
      <Icon size={14} strokeWidth={1.9} />
      {label}
    </button>
  );
}
