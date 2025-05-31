import { useSocket } from './useSocket'
import { usePresenceStore } from '~/store/presence-store'

export function usePresence(token: string) {
  const { on } = useSocket(token)
  const presenceStore = usePresenceStore()

  on('user:online', ({ userId }) => {
    presenceStore.setOnline(userId)
  })
  on('user:offline', ({ userId }) => {
    presenceStore.setOffline(userId)
  })

  return { onlineUserIds: presenceStore.onlineUserIds }
} 