import {
  archiveChannel,
  createMessage,
  deleteMessage,
  getChannel,
  getCommunity,
  listCommunityMembers,
  listMessages,
  listRoles,
  type MessageResponse,
  updateMessage,
} from '@voxnexus/api-client';
import { Archive, Eye, EyeOff, Paperclip, Send, X } from 'lucide-react';
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
import { RichText } from './RichText';
import { Tooltip } from './ui/Tooltip';

type Props = {
  channelId: string;
  onArchived?: () => void;
};

type MentionOption = {
  id: string;
  kind: 'user' | 'role' | 'everyone' | 'here';
  label: string;
  insert: string;
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

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / (1024 * 1024)).toFixed(1)} MB`;
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
    attachments: (msg.attachments ?? []).map((att) => {
      const isImage = att.content_type.startsWith('image/');
      const dims =
        att.width && att.height
          ? `${att.width}×${att.height} · ${formatBytes(att.byte_size)}`
          : formatBytes(att.byte_size);
      return {
        id: att.id,
        kind: isImage ? ('image' as const) : ('file' as const),
        name: att.filename,
        meta: dims,
        url: att.url,
        thumbnailUrl: att.thumbnail_url ?? undefined,
      };
    }),
  };
}

type PendingUpload = {
  localId: string;
  name: string;
  progress: number;
  attachmentId?: string;
  error?: string;
};

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
  const [pendingUploads, setPendingUploads] = useState<PendingUpload[]>([]);
  const [dragOver, setDragOver] = useState(false);
  const [communityId, setCommunityId] = useState<string | null>(null);
  const [mentionOptions, setMentionOptions] = useState<MentionOption[]>([]);
  const [mentionFilter, setMentionFilter] = useState<string | null>(null);
  const [mentionIndex, setMentionIndex] = useState(0);
  const [previewMarkup, setPreviewMarkup] = useState(false);
  const bottomRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
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
    setCommunityId(channel.community_id);
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
        attachments: payload.attachments ?? [],
        mentions: payload.mentions,
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
                attachments: payload.attachments ?? m.attachments,
                mentions: payload.mentions ?? m.mentions,
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
          attachments: payload.attachments ?? [],
          mentions: payload.mentions,
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

  const uploadFiles = async (files: FileList | File[]) => {
    const list = Array.from(files);
    for (const file of list) {
      const localId = crypto.randomUUID();
      setPendingUploads((prev) => [...prev, { localId, name: file.name, progress: 10 }]);
      try {
        const bytes = new Uint8Array(await file.arrayBuffer());
        setPendingUploads((prev) =>
          prev.map((p) => (p.localId === localId ? { ...p, progress: 55 } : p)),
        );
        const response = await fetch(`/api/v1/channels/${channelId}/attachments`, {
          method: 'POST',
          credentials: 'include',
          headers: {
            'Content-Type': 'application/octet-stream',
            'X-Filename': file.name,
          },
          body: bytes,
        });
        if (!response.ok) {
          const message = await response.text();
          throw new Error(message || `Upload failed (${response.status})`);
        }
        const data = (await response.json()) as { id: string };
        setPendingUploads((prev) =>
          prev.map((p) =>
            p.localId === localId ? { ...p, progress: 100, attachmentId: data.id } : p,
          ),
        );
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Upload failed';
        setPendingUploads((prev) =>
          prev.map((p) => (p.localId === localId ? { ...p, progress: 0, error: message } : p)),
        );
      }
    }
  };

  const send = async () => {
    const content = draft.trim();
    const attachmentIds = pendingUploads
      .filter((p) => p.attachmentId && !p.error)
      .map((p) => p.attachmentId as string);
    if ((!content && attachmentIds.length === 0) || sending) return;
    if (pendingUploads.some((p) => !p.attachmentId && !p.error)) return;
    setSending(true);
    const nonce = crypto.randomUUID();
    const result = await createMessage({
      path: { channel_id: channelId },
      body: {
        content,
        nonce,
        ...(replyingTo ? { referenced_message_id: replyingTo } : {}),
        ...(attachmentIds.length ? { attachment_ids: attachmentIds } : {}),
      },
    });
    setSending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not send message.'));
      return;
    }
    setDraft('');
    setPendingUploads([]);
    setReplyingTo(null);
    if (textareaRef.current) textareaRef.current.style.height = 'auto';
    setMessages((prev) => {
      if (prev.some((m) => m.id === result.data.id)) return prev;
      return [...prev, result.data];
    });
    mergeAuthors([result.data]);
    setError(null);
  };

  useEffect(() => {
    if (!communityId) return;
    void (async () => {
      const [membersResult, rolesResult] = await Promise.all([
        listCommunityMembers({ path: { community_id: communityId }, query: { limit: 100 } }),
        listRoles({ path: { community_id: communityId } }),
      ]);
      const options: MentionOption[] = [
        { id: 'everyone', kind: 'everyone', label: 'everyone', insert: '@everyone' },
        { id: 'here', kind: 'here', label: 'here', insert: '@here' },
      ];
      for (const member of membersResult.data?.items ?? []) {
        options.push({
          id: member.account_id,
          kind: 'user',
          label: member.nickname || member.display_name,
          insert: `@{${member.account_id}}`,
        });
      }
      for (const role of rolesResult.data?.roles ?? []) {
        if (role.is_everyone) continue;
        options.push({
          id: role.id,
          kind: 'role',
          label: role.name,
          insert: `@&{${role.id}}`,
        });
      }
      setMentionOptions(options);
    })();
  }, [communityId]);

  const mentionOpen = mentionFilter !== null;
  const filteredMentions =
    mentionFilter === null
      ? []
      : mentionOptions
          .filter((opt) => opt.label.toLowerCase().includes(mentionFilter.toLowerCase()))
          .slice(0, 8);

  const updateMentionFilter = (value: string, caret: number) => {
    const before = value.slice(0, caret);
    const match = /(^|\s)@([^\s@]*)$/.exec(before);
    setMentionFilter(match ? (match[2] ?? '') : null);
    setMentionIndex(0);
  };

  const applyMention = (option: MentionOption) => {
    const el = textareaRef.current;
    const caret = el?.selectionStart ?? draft.length;
    const before = draft.slice(0, caret);
    const after = draft.slice(caret);
    const replaced = before.replace(/(^|\s)@([^\s@]*)$/, `$1${option.insert} `);
    const next = `${replaced}${after}`;
    setDraft(next);
    setMentionFilter(null);
    requestAnimationFrame(() => {
      if (!el) return;
      el.focus();
      const pos = replaced.length;
      el.setSelectionRange(pos, pos);
    });
  };

  const mentionLabels = Object.fromEntries(mentionOptions.map((opt) => [opt.id, opt.label]));

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

  const uiMessages = messages.map((m) => {
    const ui = toUiMessage(m);
    const mentions = m.mentions;
    return {
      ...ui,
      mentionsMe: Boolean(
        mentions?.everyone || mentions?.here || mentions?.account_ids?.includes(accountId),
      ),
    };
  });
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
                    mentionLabels={mentionLabels}
                  />
                );
              })}
              <div ref={bottomRef} />
            </div>
          </div>

          <div
            className={`shrink-0 px-4 pb-4 pt-1 ${dragOver ? 'rounded-lg ring-2 ring-accent/50' : ''}`}
            onDragOver={(e) => {
              e.preventDefault();
              setDragOver(true);
            }}
            onDragLeave={() => setDragOver(false)}
            onDrop={(e) => {
              e.preventDefault();
              setDragOver(false);
              if (e.dataTransfer.files.length) void uploadFiles(e.dataTransfer.files);
            }}
          >
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
            {pendingUploads.length > 0 ? (
              <div
                className={`space-y-1 border border-b-0 border-line-2/60 bg-panel-2 px-3 py-2 ${
                  replyParent ? '' : 'rounded-t-lg'
                }`}
              >
                {pendingUploads.map((upload) => (
                  <div key={upload.localId} className="flex items-center gap-2 text-[12px]">
                    <span className="min-w-0 flex-1 truncate text-ink-2">{upload.name}</span>
                    {upload.error ? (
                      <span className="text-dnd">{upload.error}</span>
                    ) : (
                      <span className="font-mono text-ink-3">{upload.progress}%</span>
                    )}
                    <button
                      type="button"
                      aria-label="Remove attachment"
                      onClick={() =>
                        setPendingUploads((prev) =>
                          prev.filter((p) => p.localId !== upload.localId),
                        )
                      }
                      className="grid h-5 w-5 place-items-center rounded text-ink-3 hover:bg-surface-hover hover:text-ink"
                    >
                      <X size={12} />
                    </button>
                  </div>
                ))}
              </div>
            ) : null}
            <div className="relative">
              {previewMarkup && draft.trim() ? (
                <div className="mb-1 max-h-40 overflow-y-auto rounded-lg border border-line-2/70 bg-elevated px-3 py-2 text-[14px] leading-relaxed text-ink">
                  <p className="mb-1 font-mono text-3xs uppercase tracking-wider text-ink-4">
                    Preview
                  </p>
                  <RichText text={draft} labels={mentionLabels} />
                </div>
              ) : null}
              {mentionOpen && filteredMentions.length > 0 ? (
                <div className="absolute bottom-full left-0 right-0 z-20 mb-1 max-h-48 overflow-y-auto rounded-lg border border-line-2/70 bg-elevated shadow-panel">
                  {filteredMentions.map((item, idx) => (
                    <button
                      key={`${item.kind}-${item.id}`}
                      type="button"
                      onMouseDown={(e) => {
                        e.preventDefault();
                        applyMention(item);
                      }}
                      className={`flex w-full items-center gap-2 px-3 py-2 text-left text-sm transition-colors ${
                        idx === mentionIndex
                          ? 'bg-accent/15 text-ink'
                          : 'text-ink-2 hover:bg-surface-hover'
                      }`}
                    >
                      <span className="font-medium text-accent">{item.label}</span>
                      <span className="text-xs text-ink-4">{item.kind}</span>
                    </button>
                  ))}
                </div>
              ) : null}
              <div
                className={`group flex items-end gap-2 border border-line-2/60 bg-input px-2 py-1.5 transition-colors focus-within:border-accent/60 focus-within:shadow-accent-glow ${
                  replyParent || pendingUploads.length
                    ? 'rounded-b-lg rounded-t-none'
                    : 'rounded-lg'
                }`}
              >
                <input
                  ref={fileInputRef}
                  type="file"
                  multiple
                  className="hidden"
                  onChange={(e) => {
                    if (e.target.files?.length) void uploadFiles(e.target.files);
                    e.target.value = '';
                  }}
                />
                <Tooltip label="Attach file" side="top">
                  <button
                    type="button"
                    aria-label="Attach file"
                    onClick={() => fileInputRef.current?.click()}
                    className="mb-0.5 grid h-8 w-8 shrink-0 place-items-center rounded-md text-ink-2 transition-colors hover:bg-surface-hover hover:text-accent"
                  >
                    <Paperclip size={18} strokeWidth={2} />
                  </button>
                </Tooltip>
                <Tooltip label={previewMarkup ? 'Hide preview' : 'Preview markup'} side="top">
                  <button
                    type="button"
                    aria-label={previewMarkup ? 'Hide preview' : 'Preview markup'}
                    aria-pressed={previewMarkup}
                    onClick={() => setPreviewMarkup((v) => !v)}
                    className="mb-0.5 grid h-8 w-8 shrink-0 place-items-center rounded-md text-ink-2 transition-colors hover:bg-surface-hover hover:text-accent"
                  >
                    {previewMarkup ? (
                      <EyeOff size={18} strokeWidth={2} />
                    ) : (
                      <Eye size={18} strokeWidth={2} />
                    )}
                  </button>
                </Tooltip>
                <textarea
                  ref={textareaRef}
                  rows={1}
                  value={draft}
                  onChange={(e) => {
                    const next = e.target.value;
                    setDraft(next);
                    updateMentionFilter(next, e.target.selectionStart ?? next.length);
                    grow();
                  }}
                  onClick={(e) => {
                    updateMentionFilter(draft, e.currentTarget.selectionStart ?? draft.length);
                  }}
                  onPaste={(e) => {
                    const files = e.clipboardData?.files;
                    if (files && files.length > 0) {
                      e.preventDefault();
                      void uploadFiles(files);
                    }
                  }}
                  onKeyDown={(e) => {
                    if (mentionOpen && filteredMentions.length > 0) {
                      if (e.key === 'ArrowDown') {
                        e.preventDefault();
                        setMentionIndex((i) => (i + 1) % filteredMentions.length);
                        return;
                      }
                      if (e.key === 'ArrowUp') {
                        e.preventDefault();
                        setMentionIndex(
                          (i) => (i - 1 + filteredMentions.length) % filteredMentions.length,
                        );
                        return;
                      }
                      if (e.key === 'Enter' || e.key === 'Tab') {
                        e.preventDefault();
                        applyMention(filteredMentions[mentionIndex] ?? filteredMentions[0]!);
                        return;
                      }
                      if (e.key === 'Escape') {
                        e.preventDefault();
                        setMentionFilter(null);
                        return;
                      }
                    }
                    if (e.key === 'Enter' && !e.shiftKey) {
                      e.preventDefault();
                      void send();
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
                    disabled={
                      sending ||
                      (!draft.trim() && !pendingUploads.some((p) => p.attachmentId && !p.error)) ||
                      pendingUploads.some((p) => !p.attachmentId && !p.error)
                    }
                    onClick={() => void send()}
                    className={`mb-0.5 ml-1 grid h-8 w-8 place-items-center rounded-md transition-all ${
                      (draft.trim() || pendingUploads.some((p) => p.attachmentId)) && !sending
                        ? 'bg-accent/90 text-app hover:bg-accent'
                        : 'cursor-default bg-surface/60 text-ink-4'
                    }`}
                  >
                    <Send size={16} strokeWidth={2} />
                  </button>
                </Tooltip>
              </div>
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
