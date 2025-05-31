import { defineStore } from 'pinia'

interface ChatMessage {
  channelId: string
  content: string
  userId: string
  username: string
  createdAt: string
}

export const useChatStore = defineStore('chat-store', {
  state: () => ({
    messages: new Map<string, ChatMessage[]>(),
    isLoading: false,
  }),
  actions: {
    setMessages(channelId: string, msgs: ChatMessage[]) {
      this.messages.set(channelId, msgs)
    },
    addMessage(channelId: string, msg: ChatMessage) {
      if (!this.messages.has(channelId)) this.messages.set(channelId, [])
      this.messages.get(channelId)!.push(msg)
    },
    async fetchMessages(channelId: string) {
      // To be implemented: fetch from API or real-time
      this.setMessages(channelId, [])
    },
    editMessage(channelId: string, payload: { messageId: string; content: string; updatedAt: string }) {
      const msgs = this.messages.get(channelId)
      if (!msgs) return
      const msg = msgs.find(m => (m as any)._id === payload.messageId)
      if (msg) {
        msg.content = payload.content
        ;(msg as any).updatedAt = payload.updatedAt
      }
    },
    deleteMessage(channelId: string, payload: { messageId: string }) {
      const msgs = this.messages.get(channelId)
      if (!msgs) return
      this.messages.set(channelId, msgs.filter(m => (m as any)._id !== payload.messageId))
    },
    reactToMessage(channelId: string, payload: { messageId: string; emoji: string; userId: string; action: 'add' | 'remove' }) {
      const msgs = this.messages.get(channelId)
      if (!msgs) return
      const msg = msgs.find(m => (m as any)._id === payload.messageId)
      if (!msg) return
      if (!(msg as any).reactions) (msg as any).reactions = []
      let reaction = (msg as any).reactions.find((r: any) => r.emoji === payload.emoji)
      if (payload.action === 'add') {
        if (!reaction) {
          reaction = { emoji: payload.emoji, userIds: [] }
          ;(msg as any).reactions.push(reaction)
        }
        if (!reaction.userIds.includes(payload.userId)) reaction.userIds.push(payload.userId)
      } else if (payload.action === 'remove' && reaction) {
        reaction.userIds = reaction.userIds.filter((id: string) => id !== payload.userId)
        if (reaction.userIds.length === 0) {
          (msg as any).reactions = (msg as any).reactions.filter((r: any) => r.emoji !== payload.emoji)
        }
      }
    },
  },
}) 