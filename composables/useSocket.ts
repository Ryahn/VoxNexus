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

  // Real-time friend/block events
  function onFriendRemoved(callback: (payload: { friendId: string }) => void) {
    on('friend:removed', callback)
  }
  function onUserBlocked(callback: (payload: { blockerId: string }) => void) {
    on('user:blocked', callback)
  }

  // New real-time events
  function onFriendRequestSent(callback: (payload: { from: string, username: string }) => void) {
    on('friend:request:sent', callback)
  }
  function onFriendRequestAccepted(callback: (payload: { from: string, username?: string }) => void) {
    on('friend:request:accepted', callback)
  }
  function onFriendRequestRejected(callback: (payload: { from: string, username?: string }) => void) {
    on('friend:request:rejected', callback)
  }
  function onUserUnblocked(callback: (payload: { by: string }) => void) {
    on('user:unblocked', callback)
  }

  return { socket, on, off, emit, onFriendRemoved, onUserBlocked, onFriendRequestSent, onFriendRequestAccepted, onFriendRequestRejected, onUserUnblocked }
} 