export interface ServerToClientEvents {
  connect: () => void
  disconnect: () => void
  error: (error: Error) => void
  message: (data: any) => void
  notification: (data: any) => void
  typing: (data: any) => void
  presence: (data: any) => void
}

export interface ClientToServerEvents {
  connect: () => void
  disconnect: () => void
  message: (data: any) => void
  typing: (data: any) => void
  presence: (data: any) => void
}

export type WebSocketEventType = keyof (ServerToClientEvents & ClientToServerEvents)
export type WebSocketEventHandler = (...args: any[]) => void 