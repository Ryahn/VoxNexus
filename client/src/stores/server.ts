import { defineStore } from 'pinia'
import type { Server, ServerState, CreateServerRequest } from '@/types'
import { serverService } from '@/services/api/serverService'

export const useServerStore = defineStore('server', {
  state: (): ServerState => ({
    currentServer: null,
    servers: [],
    loading: false,
    error: null
  }),

  getters: {
    currentServerId: (state): string | null => state.currentServer?.id ?? null,
    serverList: (state): Server[] => state.servers,
    isLoading: (state): boolean => state.loading
  },

  actions: {
    async fetchServers(): Promise<void> {
      this.loading = true
      this.error = null
      try {
        const servers = await serverService.getServers()
        this.servers = servers
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to fetch servers'
        throw error
      } finally {
        this.loading = false
      }
    },

    async fetchServer(serverId: string): Promise<void> {
      this.loading = true
      this.error = null
      try {
        const server = await serverService.getServer(serverId)
        this.currentServer = server
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to fetch server'
        throw error
      } finally {
        this.loading = false
      }
    },

    async createServer(data: CreateServerRequest): Promise<void> {
      this.loading = true
      this.error = null
      try {
        const server = await serverService.createServer(data)
        this.servers.push(server)
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to create server'
        throw error
      } finally {
        this.loading = false
      }
    },

    async updateServer(serverId: string, data: Partial<Server>): Promise<void> {
      this.loading = true
      this.error = null
      try {
        const updatedServer = await serverService.updateServer(serverId, data)
        const index = this.servers.findIndex(s => s.id === serverId)
        if (index !== -1) {
          this.servers[index] = updatedServer
        }
        if (this.currentServer?.id === serverId) {
          this.currentServer = updatedServer
        }
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to update server'
        throw error
      } finally {
        this.loading = false
      }
    },

    async deleteServer(serverId: string): Promise<void> {
      this.loading = true
      this.error = null
      try {
        await serverService.deleteServer(serverId)
        this.servers = this.servers.filter(s => s.id !== serverId)
        if (this.currentServer?.id === serverId) {
          this.currentServer = null
        }
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to delete server'
        throw error
      } finally {
        this.loading = false
      }
    },

    setCurrentServer(server: Server | null): void {
      this.currentServer = server
    },

    clearError(): void {
      this.error = null
    }
  }
}) 