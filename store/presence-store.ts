import { defineStore } from 'pinia'

export const usePresenceStore = defineStore('presence-store', {
  state: () => ({
    onlineUserIds: new Set<string>(),
  }),
  actions: {
    setOnline(userId: string) {
      this.onlineUserIds.add(userId)
    },
    setOffline(userId: string) {
      this.onlineUserIds.delete(userId)
    },
    resetPresence() {
      this.onlineUserIds.clear()
    },
  },
}) 