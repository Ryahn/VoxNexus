import type { User, Message, Channel, Server, Session } from '@/types'

export type { User, Message, Channel, Server, Session }

// API Response types
export interface ApiResponse<T> {
  data: T
  status: number
  message?: string
}

// API Error type
export interface ApiError {
  status: number
  message: string
  errors?: Record<string, string[]>
}

// API Request types
export interface LoginRequest {
  email: string
  password: string
}

export interface RegisterRequest {
  username: string
  email: string
  password: string
  confirmPassword: string
}

export interface CreateMessageRequest {
  content: string
  channelId: string
}

export interface CreateChannelRequest {
  name: string
  type: 'text' | 'voice'
  serverId: string
}

// API Response types
export interface AuthResponse {
  user: User
  token: string
  refreshToken: string
  expiresIn: number // Token expiration time in seconds
}

export interface TokenResponse {
  token: string
  refreshToken: string
  expiresIn: number
}

export interface AvatarResponse {
  avatar: string
}

export interface BannerResponse {
  banner: string
}

export interface MessagesResponse {
  messages: Message[]
  hasMore: boolean
  nextCursor?: string
}

export interface CreateServerRequest {
  name: string
  icon?: File
  banner?: File
  description?: string
  isPublic?: boolean
  isNsfw?: boolean
  type?: 'public' | 'private' | 'community'
}