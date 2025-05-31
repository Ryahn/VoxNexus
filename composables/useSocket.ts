import { io, Socket } from 'socket.io-client'
import { ref } from 'vue'

const socket = ref<Socket | null>(null)

export function useSocket(token: string) {
  if (!socket.value) {
    socket.value = io('/', {
      auth: { token },
      autoConnect: true,
      transports: ['websocket'],
    })
  }

  function on(event: string, callback: (...args: any[]) => void) {
    socket.value?.on(event, callback)
  }
  function off(event: string, callback?: (...args: any[]) => void) {
    socket.value?.off(event, callback)
  }
  function emit(event: string, ...args: any[]) {
    socket.value?.emit(event, ...args)
  }

  return { socket, on, off, emit }
} 