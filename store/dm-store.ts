import { defineStore } from 'pinia'

interface DmMessage {
  from: string
  to: string
  content: string
  createdAt: string
}

export const useDmStore = defineStore('dm-store', {
  state: () => ({
    conversations: new Map<string, DmMessage[]>(),
    typing: new Set<string>(),
    isLoading: false,
  }),
  actions: {
    setMessages(userId: string, msgs: DmMessage[]) {
      this.conversations.set(userId, msgs)
    },
    addMessage(userId: string, msg: DmMessage) {
      if (!this.conversations.has(userId)) this.conversations.set(userId, [])
      this.conversations.get(userId)!.push(msg)
    },
    setTyping(userId: string) {
      this.typing.add(userId)
    },
    clearTyping(userId: string) {
      this.typing.delete(userId)
    },
    async fetchMessages(userId: string, token: string) {
      this.isLoading = true
      try {
        const res = await $fetch(`/api/dms/${userId}`, {
          headers: { Authorization: `Bearer ${token}` },
        })
        if (res && res.messages) this.setMessages(userId, res.messages)
      } finally {
        this.isLoading = false
      }
    },
  },
}) 