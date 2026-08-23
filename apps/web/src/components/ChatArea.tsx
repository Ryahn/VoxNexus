import { useUI } from '../store';
import { ChatHeader } from './ChatHeader';
import { LiveChannelPane } from './LiveChannelPane';
import { MessageComposer } from './MessageComposer';
import { MessageList } from './MessageList';

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export function ChatArea() {
  const activeChannel = useUI((s) => s.activeChannel);
  const isLiveChannel = UUID_RE.test(activeChannel);

  return (
    <main className="relative flex min-w-0 flex-1 flex-col bg-app">
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 opacity-[0.5]"
        style={{
          background:
            'radial-gradient(90% 60% at 80% -10%, rgb(var(--accent) / 0.05), transparent 55%), radial-gradient(70% 50% at 0% 110%, rgb(var(--accent-2) / 0.05), transparent 60%)',
        }}
      />
      <div className="relative flex min-h-0 flex-1 flex-col">
        {isLiveChannel ? (
          <LiveChannelPane channelId={activeChannel} />
        ) : (
          <>
            <ChatHeader />
            <MessageList />
            <MessageComposer />
          </>
        )}
      </div>
    </main>
  );
}
