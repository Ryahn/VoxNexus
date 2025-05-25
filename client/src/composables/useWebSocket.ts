import { onMounted, onUnmounted } from 'vue'
import type { WebSocketMessage } from '@/types'
import { websocketService } from '@/services/websocket/websocketService'
import type { ClientToServerEvents } from '@/types/websocket'

type WebSocketEventType = WebSocketMessage['type']
type WebSocketEventHandler = (data: any) => void

export function useWebSocket() {
  const handlers = new Map<WebSocketEventType, Set<WebSocketEventHandler>>()

  const subscribe = (event: WebSocketEventType, handler: WebSocketEventHandler): void => {
    if (!handlers.has(event)) {
      handlers.set(event, new Set())
    }
    handlers.get(event)?.add(handler)
    websocketService.subscribe(event, handler)
  }

  const unsubscribe = (event: WebSocketEventType, handler: WebSocketEventHandler): void => {
    handlers.get(event)?.delete(handler)
    websocketService.unsubscribe(event, handler)
  }

  const send = (type: keyof ClientToServerEvents, payload: any): void => {
    websocketService.send(type, payload)
  }

  onUnmounted(() => {
    // Clean up all subscriptions
    handlers.forEach((eventHandlers, event) => {
      eventHandlers.forEach(handler => {
        websocketService.unsubscribe(event, handler)
      })
    })
    handlers.clear()
  })

  return {
    subscribe,
    unsubscribe,
    send
  }
} 