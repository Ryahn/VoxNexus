import { useSocket } from './useSocket'
import { useDmStore } from '~/store/dm-store'
import { storeToRefs } from 'pinia'

export function useDm(token: string, userId: string, myId: string) {
  const { socket, on, emit } = useSocket(token)
  const dmStore = useDmStore()
  const { conversations, typing } = storeToRefs(dmStore)

  // Fetch messages
  async function fetchMessages() {
    await dmStore.fetchMessages(userId, token)
  }

  // Listen for incoming DMs
  on('dm:message', (msg: any) => {
    if ((msg.from === userId && msg.to === myId) || (msg.from === myId && msg.to === userId)) {
      dmStore.addMessage(userId, msg)
    }
  })

  // Listen for typing indicators
  on('dm:typing', ({ from }) => {
    if (from === userId) dmStore.setTyping(userId)
  })
  on('dm:stopTyping', ({ from }) => {
    if (from === userId) dmStore.clearTyping(userId)
  })

  // Send a DM
  function sendMessage(content: string) {
    const msg = {
      from: myId,
      to: userId,
      content,
      createdAt: new Date().toISOString(),
    }
    emit('dm:message', msg)
    dmStore.addMessage(userId, msg)
  }

  // Typing indicator
  function startTyping() {
    emit('dm:typing', { to: userId })
  }
  function stopTyping() {
    emit('dm:stopTyping', { to: userId })
  }

  return {
    messages: conversations.value.get(userId) || [],
    typing: typing.value.has(userId),
    fetchMessages,
    sendMessage,
    startTyping,
    stopTyping,
  }
} 