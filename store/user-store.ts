import { defineStore } from 'pinia'
import { useRouter } from 'vue-router'

interface UserProfile {
  id: string
  username: string
  email: string
}

export const useUserStore = defineStore('user-store', {
  state: () => ({
    user: null as UserProfile | null,
    allUsers: [] as any[],
  }),
  actions: {
    setUser(user: UserProfile) {
      this.user = user
    },
    clearUser() {
      this.user = null
    },
    async fetchProfile() {
      const res = await $fetch('/api/users/me')
      if (res && res.user) this.user = res.user
    },
    logout() {
      this.clearUser()
      if (process.client) {
        const router = useRouter()
        router.push('/auth/login')
      }
    },
    async fetchAllUsers() {
      try {
        const data = await $fetch('/api/users')
        this.allUsers = data?.users || []
      } catch {
        this.allUsers = []
      }
    },
  },
}) 