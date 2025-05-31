import { defineStore } from 'pinia'

interface FriendUser {
  _id: string
  username: string
  avatarUrl?: string
  bio?: string
  status?: string
}
interface FriendRequest {
  _id: string
  from?: FriendUser
  to?: FriendUser
  status: string
}

export const useFriendStore = defineStore('friend-store', {
  state: () => ({
    friends: [] as FriendUser[],
    incoming: [] as FriendRequest[],
    outgoing: [] as FriendRequest[],
    isLoading: false,
  }),
  actions: {
    async fetchFriends(token: string) {
      this.isLoading = true
      try {
        const res = await $fetch('/api/friends', {
          headers: { Authorization: `Bearer ${token}` },
        })
        if (res) {
          this.friends = (res as any).friends || []
          this.incoming = (res as any).incoming || []
          this.outgoing = (res as any).outgoing || []
        }
      } finally {
        this.isLoading = false
      }
    },
    async sendRequest(this: any, toUserId: string, token: string) {
      return await $fetch('/api/friends/request', {
        method: 'POST',
        body: { toUserId },
        headers: { Authorization: `Bearer ${token}` },
      })
    },
    async acceptRequest(this: any, requestId: string, token: string) {
      return await $fetch('/api/friends/accept', {
        method: 'POST',
        body: { requestId },
        headers: { Authorization: `Bearer ${token}` },
      })
    },
    async rejectRequest(this: any, requestId: string, token: string) {
      return await $fetch('/api/friends/reject', {
        method: 'POST',
        body: { requestId },
        headers: { Authorization: `Bearer ${token}` },
      })
    },
  },
}) 