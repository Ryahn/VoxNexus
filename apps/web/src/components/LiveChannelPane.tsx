import {
  archiveChannel,
  createMessage,
  deleteMessage,
  getChannel,
  getCommunity,
  listMessages,
  type MessageResponse,
  updateMessage,
} from '@voxnexus/api-client';
import { Archive, Send, X } from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useAuth } from '../auth';
import { apiTypeToUi } from '../lib/apiChannel';
import { readApiErrorMessage } from '../lib/apiError';
import { channelMeta } from '../lib/channelMeta';
import {
  subscribeMessageCreate,
  subscribeMessageDelete,
  subscribeMessageUpdate,
} from '../lib/gatewayMessages';
import { useUI } from '../store';
import type { Message as UiMessage, User } from '../types';
import { Message } from './Message';
import { Tooltip } from './ui/Tooltip';

type Props = {
  channelId: string;
  onArchived?: () => void;
};

function authorFromApi(msg: MessageResponse): User {
  return {
    id: msg.author_id,
    displayName: msg.author_display_name,
    username: msg.author_display_name,
    avatarSeed: msg.author_id,
    accent: '141 152 173',
    presence: 'online',
    roleIds: [],
  };
}

function toUiMessage(msg: MessageResponse): UiMessage {
  const created = new Date(msg.created_at);
  return {
    id: msg.id,
    channelId: msg.channel_id,
    authorId: msg.author_id,
    ts: created.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
    fullTs: created.toLocaleString(),
    content: msg.content,
    edited: Boolean(msg.edited_at),
    replyTo: msg.reply_to
      ? {
          messageId: msg.reply_to.message_id,
          authorId: msg.reply_to.author_id,
          excerpt: msg.reply_to.excerpt,
          authorDisplayName: msg.reply_to.author_display_name,
          deleted: msg.reply_to.deleted,
        }
      : undefined,
  };
}

export function LiveChannelPane({ channelId, onArchived }: Props) {
  const { session } = useAuth();
  const setChannel = useUI((s) => s.setChannel);
  const replyingTo = useUI((s) => s.replyingTo);
  const setReplyingTo = useUI((s) => s.setReplyingTo);
  const [name, setName] = useState('Channel');
  const [topic, setTopic] = useState('');
  const [apiType, setApiType] = useState('text');
  const [isOwner, setIsOwner] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [messages, setMessages] = useState<MessageResponse[]>([]);
  const [authors, setAuthors] = useState<Record<string, User>>({});
  const [draft, setDraft] = useState('');
  const [sending, setSending] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editDraft, setEditDraft] = useState('');
  const [editPending, setEditPending] = useState(false);
  const bottomRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const accountId = session.account.id;

  const mergeAuthors = useCallback((items: MessageResponse[]) => {
    setAuthors((prev) => {
      const next = { ...prev };
      for (const item of items) {
        next[item.author_id] = authorFromApi(item);
      }
      return next;
    });
  }, []);

  const refreshChannel = useCallback(async () => {
    const result = await getChannel({ path: { channel_id: channelId } });
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not load channel.'));
      return;
    }
    const channel = result.data;
    setName(channel.name);
    setTopic(channel.topic);
    setApiType(channel.type);
    setError(null);

    const communityResult = await getCommunity({
      path: { community_id: channel.community_id },
    });
    setIsOwner(communityResult.data?.owner_account_id === session.account.id);
  }, [channelId, session.account.id]);

  const refreshMessages = useCallback(async () => {
    const result = await listMessages({
      path: { channel_id: channelId },
      query: { limit: 50 },
    });
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not load messages.'));
      return;
    }
    // API returns newest first; UI shows oldest → newest.
    const chronological = [...result.data.items].reverse();
    setMessages(chronological);
    mergeAuthors(chronological);
    setError(null);
  }, [channelId, mergeAuthors]);

  useEffect(() => {
    void refreshChannel();
  }, [refreshChannel]);

  useEffect(() => {
    setReplyingTo(null);
  }, [channelId, setReplyingTo]);

  useEffect(() => {
    if (apiType !== 'text') return;
    void refreshMessages();
  }, [apiType, refreshMessages]);

  useEffect(() => {
    if (apiType !== 'text') return;
    const unsubCreate = subscribeMessageCreate((payload) => {
      if (payload.channel_id !== channelId) return;
      const incoming: MessageResponse = {
        id: payload.id,
        channel_id: payload.channel_id,
        community_id: payload.community_id,
        author_id: payload.author_id,
        author_display_name: payload.author_display_name,
        content: payload.content,
        nonce: payload.nonce,
        referenced_message_id: payload.referenced_message_id,
        reply_to: payload.reply_to,
        created_at: payload.created_at,
        edited_at: payload.edited_at,
      };
      setMessages((prev) => {
        if (prev.some((m) => m.id === incoming.id)) return prev;
        return [...prev, incoming];
      });
      mergeAuthors([incoming]);
    });
    const unsubUpdate = subscribeMessageUpdate((payload) => {
      if (payload.channel_id !== channelId) return;
      setMessages((prev) =>
        prev.map((m) =>
          m.id === payload.id
            ? {
                ...m,
                content: payload.content,
                edited_at: payload.edited_at,
                author_display_name: payload.author_display_name,
                referenced_message_id: payload.referenced_message_id,
                reply_to: payload.reply_to,
              }
            : m,
        ),
      );
      mergeAuthors([
        {
          id: payload.id,
          channel_id: payload.channel_id,
          community_id: payload.community_id,
          author_id: payload.author_id,
          author_display_name: payload.author_display_name,
          content: payload.content,
          nonce: payload.nonce,
          referenced_message_id: payload.referenced_message_id,
          reply_to: payload.reply_to,
          created_at: payload.created_at,
          edited_at: payload.edited_at,
        },
      ]);
    });
    const unsubDelete = subscribeMessageDelete((payload) => {
      if (payload.channel_id !== channelId) return;
      setMessages((prev) => prev.filter((m) => m.id !== payload.id));
      setEditingId((id) => (id === payload.id ? null : id));
    });
    return () => {
      unsubCreate();
      unsubUpdate();
      unsubDelete();
    };
  }, [apiType, channelId, mergeAuthors]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const uiType = apiTypeToUi(apiType);
  const { Icon, label } = channelMeta[uiType] ?? channelMeta.text;
  const isText = apiType === 'text';

  const archive = async () => {
    if (!isOwner || pending) return;
    setPending(true);
    const result = await archiveChannel({ path: { channel_id: channelId } });
    setPending(false);
    if (result.error) {
      setError(readApiErrorMessage(result.error, 'Could not archive channel.'));
      return;
    }
    onArchived?.();
    setChannel(channelId);
  };

  const grow = () => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = `${Math.min(el.scrollHeight, 220)}px`;
  };

  const send = async () => {
    const content = draft.trim();
    if (!content || sending) return;
    setSending(true);
    const nonce = crypto.randomUUID();
    const result = await createMessage({
      path: { channel_id: channelId },
      body: {
        content,
        nonce,
        ...(replyingTo ? { referenced_message_id: replyingTo } : {}),
      },
    });
    setSending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not send message.'));
      return;
    }
    setDraft('');
    setReplyingTo(null);
    if (textareaRef.current) textareaRef.current.style.height = 'auto';
    setMessages((prev) => {
      if (prev.some((m) => m.id === result.data.id)) return prev;
      return [...prev, result.data];
    });
    mergeAuthors([result.data]);
    setError(null);
  };

  const jumpToReply = (messageId: string) => {
    const el = document.getElementById(`msg-${messageId}`);
    el?.scrollIntoView({ behavior: 'smooth', block: 'center' });
  };

  const startEdit = (msg: MessageResponse) => {
    setEditingId(msg.id);
    setEditDraft(msg.content);
  };

  const saveEdit = async () => {
    if (!editingId || editPending) return;
    const content = editDraft.trim();
    if (!content) return;
    setEditPending(true);
    const result = await updateMessage({
      path: { channel_id: channelId, message_id: editingId },
      body: { content },
    });
    setEditPending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not edit message.'));
      return;
    }
    setMessages((prev) => prev.map((m) => (m.id === result.data.id ? result.data : m)));
    setEditingId(null);
    setError(null);
  };

  const removeMessage = async (messageId: string) => {
    const result = await deleteMessage({
      path: { channel_id: channelId, message_id: messageId },
    });
    if (result.error) {
      setError(readApiErrorMessage(result.error, 'Could not delete message.'));
      return;
    }
    setMessages((prev) => prev.filter((m) => m.id !== messageId));
    setEditingId((id) => (id === messageId ? null : id));
    setError(null);
  };

  const uiMessages = messages.map(toUiMessage);
  const replyParent = replyingTo ? messages.find((m) => m.id === replyingTo) : null;

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <header className="flex h-12 shrink-0 items-center gap-2 border-b border-line/70 px-4">
        <Icon size={18} className="shrink-0 text-ink-3" strokeWidth={1.9} />
        <div className="min-w-0 flex-1">
          <h1 className="truncate text-[15px] font-semibold text-ink">{name}</h1>
          {topic ? <p className="truncate text-xs text-ink-3">{topic}</p> : null}
        </div>
        {isOwner ? (
          <button
            type="button"
            title="Archive channel"
            disabled={pending}
            onClick={() => void archive()}
            className="rounded p-1.5 text-ink-3 hover:bg-surface-hover hover:text-ink disabled:opacity-50"
          >
            <Archive size={16} />
          </button>
        ) : null}
      </header>

      {isText ? (
        <>
          <div className="min-h-0 flex-1 overflow-y-auto overflow-x-hidden overscroll-contain">
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
                Welcome to <span className="text-accent">#{name}</span>
              </h2>
              <p className="mt-1 max-w-xl text-[13.5px] leading-relaxed text-ink-2">
                {topic || `This is the start of #${name}.`}
              </p>
            </div>

            {error ? <p className="px-4 pb-2 text-sm text-dnd">{error}</p> : null}

            <div className="pb-3">
              {uiMessages.map((m, i) => {
                const prev = uiMessages[i - 1];
                const grouped = Boolean(
                  prev && prev.authorId === m.authorId && !m.replyTo && !m.pinned,
                );
                const raw = messages[i];
                const mine = raw?.author_id === accountId;
                if (editingId === m.id && raw) {
                  return (
                    <div key={m.id} className="px-4 py-2">
                      <textarea
                        value={editDraft}
                        onChange={(e) => setEditDraft(e.target.value)}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter' && !e.shiftKey) {
                            e.preventDefault();
                            void saveEdit();
                          }
                          if (e.key === 'Escape') setEditingId(null);
                        }}
                        className="w-full resize-y rounded-lg border border-line-2/60 bg-input px-3 py-2 font-body text-[14px] text-ink outline-none focus:border-accent/60"
                        rows={3}
                      />
                      <div className="mt-1.5 flex gap-2">
                        <button
                          type="button"
                          disabled={editPending || !editDraft.trim()}
                          onClick={() => void saveEdit()}
                          className="rounded-md bg-accent/90 px-2.5 py-1 text-xs font-medium text-app hover:bg-accent disabled:opacity-50"
                        >
                          Save
                        </button>
                        <button
                          type="button"
                          onClick={() => setEditingId(null)}
                          className="rounded-md px-2.5 py-1 text-xs text-ink-3 hover:bg-surface-hover hover:text-ink"
                        >
                          Cancel
                        </button>
                      </div>
                    </div>
                  );
                }
                return (
                  <Message
                    key={m.id}
                    message={m}
                    grouped={grouped}
                    author={authors[m.authorId]}
                    canEdit={mine}
                    canDelete={mine || isOwner}
                    onEdit={raw ? () => startEdit(raw) : undefined}
                    onDelete={raw ? () => void removeMessage(raw.id) : undefined}
                    onJumpToReply={jumpToReply}
                  />
                );
              })}
              <div ref={bottomRef} />
            </div>
          </div>

          <div className="shrink-0 px-4 pb-4 pt-1">
            {replyParent ? (
              <div className="flex items-center gap-2 rounded-t-lg border border-b-0 border-line-2/60 bg-panel-2 px-3 py-1.5 text-[12px]">
                <span className="text-ink-3">Replying to</span>
                <span className="font-semibold text-ink">{replyParent.author_display_name}</span>
                <span className="truncate text-ink-3">{replyParent.content}</span>
                <button
                  type="button"
                  aria-label="Cancel reply"
                  onClick={() => setReplyingTo(null)}
                  className="ml-auto grid h-5 w-5 place-items-center rounded text-ink-3 hover:bg-surface-hover hover:text-ink"
                >
                  <X size={13} />
                </button>
              </div>
            ) : null}
            <div
              className={`group flex items-end gap-2 border border-line-2/60 bg-input px-2 py-1.5 transition-colors focus-within:border-accent/60 focus-within:shadow-accent-glow ${
                replyParent ? 'rounded-b-lg rounded-t-none' : 'rounded-lg'
              }`}
            >
              <textarea
                ref={textareaRef}
                rows={1}
                value={draft}
                onChange={(e) => {
                  setDraft(e.target.value);
                  grow();
                }}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    if (draft.trim()) void send();
                  }
                  if (e.key === 'Escape' && replyingTo) {
                    e.preventDefault();
                    setReplyingTo(null);
                  }
                }}
                placeholder={
                  replyParent ? `Reply to ${replyParent.author_display_name}` : `Message #${name}`
                }
                className="my-1 max-h-[220px] min-h-[24px] flex-1 resize-none bg-transparent font-body text-[14px] leading-relaxed text-ink outline-none placeholder:text-ink-4"
              />
              <Tooltip label="Send message" side="top">
                <button
                  type="button"
                  aria-label="Send message"
                  disabled={!draft.trim() || sending}
                  onClick={() => void send()}
                  className={`mb-0.5 ml-1 grid h-8 w-8 place-items-center rounded-md transition-all ${
                    draft.trim() && !sending
                      ? 'bg-accent/90 text-app hover:bg-accent'
                      : 'cursor-default bg-surface/60 text-ink-4'
                  }`}
                >
                  <Send size={16} strokeWidth={2} />
                </button>
              </Tooltip>
            </div>
          </div>
        </>
      ) : (
        <div className="flex min-h-0 flex-1 flex-col items-center justify-center px-6 text-center">
          {error ? (
            <p className="text-sm text-dnd">{error}</p>
          ) : (
            <>
              <p className="text-sm font-medium text-ink-2">{label} channels</p>
              <p className="mt-1 max-w-sm text-sm text-ink-3">
                {label} features are coming in a later milestone.
              </p>
            </>
          )}
        </div>
      )}
    </div>
  );
}
