import { ChatHeader } from './ChatHeader';
import { MessageComposer } from './MessageComposer';
import { MessageList } from './MessageList';

export function ChatArea() {
  return (
    <main className="relative flex min-w-0 flex-1 flex-col bg-app">
      {/* very faint atmospheric wash behind the chat */}
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 opacity-[0.5]"
        style={{
          background:
            'radial-gradient(90% 60% at 80% -10%, rgb(var(--accent) / 0.05), transparent 55%), radial-gradient(70% 50% at 0% 110%, rgb(var(--accent-2) / 0.05), transparent 60%)',
        }}
      />
      <div className="relative flex min-h-0 flex-1 flex-col">
        <ChatHeader />
        <MessageList />
        <MessageComposer />
      </div>
    </main>
  );
}
