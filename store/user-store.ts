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
    token: '' as string,
    allUsers: [] as any[],
  }),
  actions: {
    setUser(user: UserProfile, token: string) {
      this.user = user
      this.token = token
    },
    clearUser() {
      this.user = null
      this.token = ''
    },
    async fetchProfile() {
      if (!this.token) return
      const res = await $fetch('/api/users/me', {
        headers: { Authorization: `Bearer ${this.token}` },
      })
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
        const { data } = await useFetch('/api/users')
        this.allUsers = data.value?.users || []
      } catch {
        this.allUsers = []
      }
    },
  },
}) 