import { defineStore } from 'pinia'

interface Channel {
  id: string
  name: string
  type: string
  createdAt: string
}

interface Server {
  id: string
  name: string
  ownerId: string
  createdAt: string
  channels: Channel[]
}

export const useServerStore = defineStore('server-store', {
  state: () => ({
    servers: [] as Server[],
    currentServerId: '' as string,
    currentChannelId: '' as string,
  }),
  actions: {
    async fetchServers(token: string) {
      const res = await $fetch('/api/servers', {
        headers: { Authorization: `Bearer ${token}` },
      })
      if (res && res.servers) this.servers = res.servers
    },
    setCurrentServer(serverId: string) {
      this.currentServerId = serverId
    },
    setCurrentChannel(channelId: string) {
      this.currentChannelId = channelId
    },
    async fetchChannels(serverId: string, token: string) {
      const res = await $fetch(`/api/servers/${serverId}/channels`, {
        headers: { Authorization: `Bearer ${token}` },
      })
      const server = this.servers.find(s => s.id === serverId)
      if (server && res && res.channels) server.channels = res.channels
    },
  },
}) 