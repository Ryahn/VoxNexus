import { apiClient } from './client'
import type { User, AuthResponse, LoginRequest, RegisterRequest, AvatarResponse, BannerResponse, Session } from './types'

export const authService = {
  async login(email: string, password: string): Promise<AuthResponse> {
    const loginData: LoginRequest = { email, password }
    const response = await apiClient.post<AuthResponse>('/auth/login', loginData)
    return response.data
  },

  async register(username: string, email: string, password: string, confirmPassword: string): Promise<AuthResponse> {
    console.log('[AUTH_SERVICE] Starting registration API call', { username, email })
    const registerData: RegisterRequest = { username, email, password, confirmPassword }
    console.log('[AUTH_SERVICE] Making POST request to /auth/register')
    const response = await apiClient.post<AuthResponse>('/auth/register', registerData)
    console.log('[AUTH_SERVICE] Registration API response received', { status: response.status })
    return response.data
  },

  async getCurrentUser(): Promise<User> {
    const response = await apiClient.get<User>('/auth/me')
    return response.data
  },

  async logout(): Promise<void> {
    await apiClient.post('/auth/logout')
  },

  async refreshToken(): Promise<AuthResponse> {
    const refreshToken = localStorage.getItem('refreshToken')
    if (!refreshToken) {
      throw new Error('No refresh token available')
    }
    const response = await apiClient.post<AuthResponse>('/auth/refresh', { refreshToken })
    return response.data
  },

  async getSessions(): Promise<Session[]> {
    const response = await apiClient.get<{ sessions: Session[] }>('/auth/sessions')
    return response.data.sessions
  },

  async updatePassword(currentPassword: string, newPassword: string): Promise<void> {
    await apiClient.put('/auth/password', { currentPassword, newPassword })
  },

  async updateProfile(data: { username: string; email: string }): Promise<User> {
    const response = await apiClient.put<User>('/auth/profile', data)
    return response.data
  },

  async updateAvatar(file: File): Promise<User> {
    const formData = new FormData()
    formData.append('avatar', file)
    await apiClient.post<AvatarResponse>('/auth/avatar', formData, {
      headers: {
        'Content-Type': 'multipart/form-data'
      }
    })
    return this.getCurrentUser()
  },

  async updateBanner(file: File): Promise<User> {
    const formData = new FormData()
    formData.append('banner', file)
    await apiClient.post<BannerResponse>('/auth/banner', formData, {
      headers: {
        'Content-Type': 'multipart/form-data'
      }
    })
    return this.getCurrentUser()
  },

  async logoutSession(sessionId: string): Promise<void> {
    await apiClient.delete(`/auth/sessions/${sessionId}`)
  },

  async logoutAllSessions(): Promise<void> {
    await apiClient.delete('/auth/sessions')
  },

  async requestPasswordReset(email: string): Promise<void> {
    await apiClient.post('/auth/forgot-password', { email })
  },

  async verifyPasswordResetToken(token: string): Promise<void> {
    await apiClient.get(`/auth/verify-reset-token/${token}`)
  },

  async resetPassword(token: string, newPassword: string): Promise<void> {
    await apiClient.post('/auth/reset-password', { token, newPassword })
  }
} 