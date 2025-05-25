import { io, Socket } from 'socket.io-client'
import type { ServerToClientEvents, ClientToServerEvents } from '@/types/websocket'
import type { WebSocketEventType, WebSocketEventHandler } from '@/types/websocket'

class WebSocketService {
  private static instance: WebSocketService
  private socket: Socket<ServerToClientEvents, ClientToServerEvents> | null = null
  private readonly baseUrl: string

  private constructor() {
    this.baseUrl = process.env.VUE_APP_WS_URL || 'ws://localhost:3000'
  }

  static getInstance(): WebSocketService {
    if (!WebSocketService.instance) {
      WebSocketService.instance = new WebSocketService()
    }
    return WebSocketService.instance
  }

  connect(token: string): void {
    if (this.socket?.connected) return

    this.socket = io(this.baseUrl, {
      auth: { token },
      transports: ['websocket']
    })

    this.socket.on('connect', () => {
      console.log('WebSocket connected')
    })

    this.socket.on('disconnect', () => {
      console.log('WebSocket disconnected')
    })

    this.socket.on('error', (error: Error) => {
      console.error('WebSocket error:', error)
    })
  }

  disconnect(): void {
    if (this.socket) {
      this.socket.disconnect()
      this.socket = null
    }
  }

  getSocket(): Socket<ServerToClientEvents, ClientToServerEvents> | null {
    return this.socket
  }

  subscribe(event: WebSocketEventType, handler: WebSocketEventHandler): void {
    if (this.socket) {
      this.socket.on(event, handler)
    }
  }

  unsubscribe(event: WebSocketEventType, handler: WebSocketEventHandler): void {
    if (this.socket) {
      this.socket.off(event, handler)
    }
  }

  send(type: keyof ClientToServerEvents, payload: any): void {
    if (this.socket) {
      this.socket.emit(type, payload)
    }
  }
}

export const websocketService = WebSocketService.getInstance() 