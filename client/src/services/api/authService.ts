import { apiClient } from './client'
import type { User, AuthResponse, LoginRequest, RegisterRequest, AvatarResponse, BannerResponse, Session } from './types'

export const authService = {
  async login(email: string, password: string): Promise<AuthResponse> {
    const loginData: LoginRequest = { email, password }
    const response = await apiClient.post<AuthResponse>('/auth/login', loginData)
    return response.data
  },

  async register(username: string, email: string, password: string, confirmPassword: string): Promise<AuthResponse> {
    const registerData: RegisterRequest = { username, email, password, confirmPassword }
    const response = await apiClient.post<AuthResponse>('/auth/register', registerData)
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
    const response = await apiClient.post<AuthResponse>('/auth/refresh')
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
  }
} 