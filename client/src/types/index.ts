// API Error type
export interface ApiError {
  status: number
  message: string
  errors?: Record<string, string[]>
}

// User related types
export interface User {
  id: string
  username: string
  email: string
  avatar?: string
  banner?: string
  status?: 'online' | 'offline' | 'idle' | 'dnd'
  lastSeen?: Date
}

// Server related types
export interface Server {
  id: string
  name: string
  icon?: string
  ownerId: string
  channels: Channel[]
  members: ServerMember[]
  createdAt: Date
  updatedAt: Date
}

export interface ServerMember {
  id: string
  userId: string
  serverId: string
  roles: string[]
  nickname?: string
  joinedAt: Date
}

// Channel related types
export interface Channel {
  id: string
  name: string
  type: 'text' | 'voice'
  serverId: string
  position: number
  parentId?: string
  createdAt: Date
  updatedAt: Date
}

// Message related types
export interface Message {
  id: string
  content: string
  userId: string
  channelId: string
  attachments?: MessageAttachment[]
  mentions?: string[]
  createdAt: Date
  updatedAt: Date
}

export interface MessageAttachment {
  id: string
  type: 'image' | 'file' | 'video' | 'audio'
  url: string
  name: string
  size: number
}

// Role related types
export interface Role {
  id: string
  name: string
  color: string
  permissions: string[]
  position: number
  serverId: string
}

// WebSocket related types
export interface WebSocketMessage {
  type: 'message' | 'typing' | 'presence' | 'error'
  payload: any
}

// API Response types
export interface PaginatedResponse<T> {
  items: T[]
  total: number
  page: number
  limit: number
  hasMore: boolean
  nextCursor?: string
}

// State management types
export interface AuthState {
  user: User | null
  token: string | null
  refreshToken: string | null
  tokenExpiry: Date | null
  isAuthenticated: boolean
}

export interface ServerState {
  currentServer: Server | null
  servers: Server[]
  loading: boolean
  error: string | null
}

export interface ChannelState {
  currentChannel: Channel | null
  channels: Channel[]
  loading: boolean
  error: string | null
}

export interface MessageState {
  messages: Message[]
  loading: boolean
  error: string | null
  hasMore: boolean
  nextCursor?: string
}

export interface Session {
  id: string
  device: string
  lastActive: Date
  current: boolean
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