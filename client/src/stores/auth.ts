import { defineStore } from 'pinia'
import type { User, AuthState } from '@/types'
import { authService } from '@/services/api/authService'
import { websocketService } from '@/services/websocket/websocketService'

export const useAuthStore = defineStore('auth', {
  state: (): AuthState => ({
    user: null,
    token: localStorage.getItem('token'),
    isAuthenticated: false
  }),

  getters: {
    currentUser: (state) => state.user,
    isLoggedIn: (state) => state.isAuthenticated
  },

  actions: {
    async login(email: string, password: string): Promise<void> {
      const { user, token } = await authService.login(email, password)
      this.setAuth(user, token)
      websocketService.connect(token)
    },

    async register(username: string, email: string, password: string, confirmPassword: string): Promise<void> {
      try {
        const { user, token } = await authService.register(username, email, password, confirmPassword)
        this.setAuth(user, token)
        websocketService.connect(token)
      } catch (err: any) {
        // Propagate backend error message
        throw err?.response?.data?.error || err?.message || 'Registration failed'
      }
    },

    async logout(): Promise<void> {
      await authService.logout()
      this.clearAuth()
      websocketService.disconnect()
    },

    async fetchCurrentUser(): Promise<void> {
      const user = await authService.getCurrentUser()
      this.user = user
      this.isAuthenticated = true
    },

    setAuth(user: User, token: string): void {
      this.user = user
      this.token = token
      this.isAuthenticated = true
      localStorage.setItem('token', token)
    },

    clearAuth(): void {
      this.user = null
      this.token = null
      this.isAuthenticated = false
      localStorage.removeItem('token')
    },

    async getSessions(): Promise<Array<{ id: string; device: string; lastActive: Date }>> {
      return await authService.getSessions()
    },

    async updatePassword(currentPassword: string, newPassword: string): Promise<void> {
      await authService.updatePassword(currentPassword, newPassword)
    },

    async updateProfile(data: { username: string; email: string }): Promise<void> {
      const user = await authService.updateProfile(data)
      this.user = user
    },

    async updateAvatar(file: File): Promise<void> {
      const user = await authService.updateAvatar(file)
      this.user = user
    },

    async updateBanner(file: File): Promise<void> {
      const user = await authService.updateBanner(file)
      this.user = user
    },

    async logoutSession(sessionId: string): Promise<void> {
      await authService.logoutSession(sessionId)
    },

    async logoutAllSessions(): Promise<void> {
      await authService.logoutAllSessions()
      this.clearAuth()
      websocketService.disconnect()
    }
  }
}) 