import { defineStore } from 'pinia'
import type { User, AuthState } from '@/types'
import { authService } from '@/services/api/authService'
import { websocketService } from '@/services/websocket/websocketService'

export const useAuthStore = defineStore('auth', {
  state: (): AuthState => ({
    user: null,
    token: localStorage.getItem('token'),
    refreshToken: localStorage.getItem('refreshToken'),
    tokenExpiry: localStorage.getItem('tokenExpiry') ? new Date(localStorage.getItem('tokenExpiry')!) : null,
    isAuthenticated: false
  }),

  getters: {
    currentUser: (state) => state.user,
    isLoggedIn: (state) => state.isAuthenticated,
    isTokenExpired: (state) => {
      if (!state.tokenExpiry) return true
      return new Date() >= state.tokenExpiry
    }
  },

  actions: {
    async login(email: string, password: string): Promise<void> {
      const { user, token, refreshToken, expiresIn } = await authService.login(email, password)
      this.setAuth(user, token, refreshToken, expiresIn)
      websocketService.connect(token)
    },

    async register(username: string, email: string, password: string, confirmPassword: string): Promise<void> {
      try {
        console.log('[AUTH_STORE] Starting registration', { username, email })
        const { user, token, refreshToken, expiresIn } = await authService.register(username, email, password, confirmPassword)
        console.log('[AUTH_STORE] Registration API call successful', { user, token: token ? 'present' : 'missing' })
        this.setAuth(user, token, refreshToken, expiresIn)
        websocketService.connect(token)
        console.log('[AUTH_STORE] Registration completed successfully')
      } catch (err: any) {
        console.error('[AUTH_STORE] Registration error:', err)
        // Handle both ApiError (from our client) and axios errors
        const errorMessage = err?.message || err?.response?.data?.error || 'Registration failed'
        throw new Error(errorMessage)
      }
    },

    async logout(): Promise<void> {
      await authService.logout()
      this.clearAuth()
      websocketService.disconnect()
    },

    async refreshTokens(): Promise<void> {
      if (!this.refreshToken) {
        throw new Error('No refresh token available')
      }

      const { token, refreshToken, expiresIn } = await authService.refreshToken()
      this.setTokens(token, refreshToken, expiresIn)
      websocketService.connect(token)
    },

    async fetchCurrentUser(): Promise<void> {
      const user = await authService.getCurrentUser()
      this.user = user
      this.isAuthenticated = true
    },

    setAuth(user: User, token: string, refreshToken: string, expiresIn: number): void {
      this.user = user
      this.setTokens(token, refreshToken, expiresIn)
      this.isAuthenticated = true
    },

    setTokens(token: string, refreshToken: string, expiresIn: number): void {
      this.token = token
      this.refreshToken = refreshToken
      const expiryDate = new Date()
      expiryDate.setSeconds(expiryDate.getSeconds() + expiresIn)
      this.tokenExpiry = expiryDate

      localStorage.setItem('token', token)
      localStorage.setItem('refreshToken', refreshToken)
      localStorage.setItem('tokenExpiry', expiryDate.toISOString())
    },

    clearAuth(): void {
      this.user = null
      this.token = null
      this.refreshToken = null
      this.tokenExpiry = null
      this.isAuthenticated = false
      localStorage.removeItem('token')
      localStorage.removeItem('refreshToken')
      localStorage.removeItem('tokenExpiry')
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
    },

    async requestPasswordReset(email: string): Promise<void> {
      await authService.requestPasswordReset(email)
    },

    async verifyPasswordResetToken(token: string): Promise<void> {
      await authService.verifyPasswordResetToken(token)
    },

    async resetPassword(token: string, newPassword: string): Promise<void> {
      await authService.resetPassword(token, newPassword)
    }
  }
}) 