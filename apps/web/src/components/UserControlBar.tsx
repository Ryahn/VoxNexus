import { HeadphoneOff, Headphones, Mic, MicOff, Settings } from 'lucide-react';
import { me } from '../data/users';
import { presenceLabel, usePresence } from '../presence';
import { useUI } from '../store';
import { Avatar } from './ui/Avatar';
import { IconButton } from './ui/IconButton';
import { PresenceDot } from './ui/Presence';

export function UserControlBar() {
  const muted = useUI((s) => s.muted);
  const deafened = useUI((s) => s.deafened);
  const toggleMute = useUI((s) => s.toggleMute);
  const toggleDeafen = useUI((s) => s.toggleDeafen);
  const openProfile = useUI((s) => s.openProfile);
  const setSettingsOpen = useUI((s) => s.setSettingsOpen);
  const { self } = usePresence();
  const statusLine = presenceLabel(self?.status ?? 'online', self?.customStatus ?? me.status);

  return (
    <div className="flex items-center gap-1 border-t border-line/70 bg-panel-2 px-1.5 py-1.5">
      <button
        type="button"
        onClick={(e) => {
          const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
          openProfile('me', { x: r.left, y: r.top - 8, bottom: true });
        }}
        className="group flex min-w-0 flex-1 items-center gap-2 rounded-md px-1.5 py-1 text-left transition-colors hover:bg-surface-hover"
      >
        <div className="relative">
          <Avatar
            user={me}
            size={30}
            rounded="rounded-full"
            showPresence={false}
            ring="rgb(var(--bg-panel-2))"
          />
          {self && (
            <span className="absolute bottom-0 right-0">
              <PresenceDot
                presence={self.status === 'invisible' ? 'offline' : self.status}
                size={10}
                ring="rgb(var(--bg-panel-2))"
              />
            </span>
          )}
        </div>
        <span className="min-w-0 flex-1 leading-tight">
          <span className="block truncate text-[12.5px] font-semibold text-ink">
            {me.displayName}
          </span>
          <span className="block truncate font-mono text-3xs text-ink-3">{statusLine}</span>
        </span>
      </button>
      <div className="flex items-center">
        <IconButton
          icon={muted ? MicOff : Mic}
          label={muted ? 'Unmute' : 'Mute'}
          tipSide="top"
          onClick={toggleMute}
          className={muted ? '!text-dnd' : ''}
        />
        <IconButton
          icon={deafened ? HeadphoneOff : Headphones}
          label={deafened ? 'Undeafen' : 'Deafen'}
          tipSide="top"
          onClick={toggleDeafen}
          className={deafened ? '!text-dnd' : ''}
        />
        <IconButton
          icon={Settings}
          label="User Settings"
          tipSide="top"
          kbd="Ctrl ,"
          onClick={() => setSettingsOpen(true)}
        />
      </div>
    </div>
  );
}
