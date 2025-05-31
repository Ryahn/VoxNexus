import { useSocket } from './useSocket'
import { useChatStore } from '~/store/chat-store'
import { storeToRefs } from 'pinia'
import type {
  ChannelMessageEditPayload,
  ChannelMessageDeletePayload,
  ChannelMessageReactionPayload,
  ChannelEvents
} from '~/types/socket-events';

export function useChat(token: string, channelId: string) {
  const { socket, on, emit } = useSocket(token)
  const chatStore = useChatStore()
  const { messages } = storeToRefs(chatStore)

  // Join channel room
  emit('channel:join', channelId)

  // Listen for incoming messages
  on('channel:message', (msg: any) => {
    if (msg.channelId === channelId) {
      chatStore.addMessage(channelId, msg)
    }
  })
  on('channel:edit-message', (payload: ChannelMessageEditPayload) => {
    chatStore.editMessage(channelId, payload)
  })
  on('channel:delete-message', (payload: ChannelMessageDeletePayload) => {
    chatStore.deleteMessage(channelId, payload)
  })
  on('channel:reaction', (payload: ChannelMessageReactionPayload) => {
    chatStore.reactToMessage(channelId, payload)
  })

  // Send a message
  function sendMessage(content: string, userId: string, username: string) {
    const msg = {
      channelId,
      content,
      userId,
      username,
      createdAt: new Date().toISOString(),
    }
    emit('channel:message', msg)
    chatStore.addMessage(channelId, msg)
  }
  function editMessage(payload: ChannelMessageEditPayload) {
    emit('channel:edit-message', payload)
    chatStore.editMessage(channelId, payload)
  }
  function deleteMessage(payload: ChannelMessageDeletePayload) {
    emit('channel:delete-message', payload)
    chatStore.deleteMessage(channelId, payload)
  }
  function reactToMessage(payload: ChannelMessageReactionPayload) {
    emit('channel:reaction', payload)
    chatStore.reactToMessage(channelId, payload)
  }

  // Leave channel room on cleanup
  function leaveChannel() {
    emit('channel:leave', channelId)
  }

  return {
    messages,
    sendMessage,
    editMessage,
    deleteMessage,
    reactToMessage,
    leaveChannel,
  }
} 