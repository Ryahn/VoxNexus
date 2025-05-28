import { apiClient } from './client'
import type { Server, ServerMember, Role, CreateServerRequest } from '@/types'

export class ServerService {
  private static instance: ServerService
  private readonly baseUrl = '/api/servers'

  // eslint-disable-next-line @typescript-eslint/no-empty-function
  private constructor() {}

  static getInstance(): ServerService {
    if (!ServerService.instance) {
      ServerService.instance = new ServerService()
    }
    return ServerService.instance
  }

  async getServers(): Promise<Server[]> {
    const response = await apiClient.get<Server[]>(this.baseUrl)
    return response.data
  }

  async getServer(serverId: string): Promise<Server> {
    const response = await apiClient.get<Server>(`${this.baseUrl}/${serverId}`)
    return response.data
  }

  async createServer(data: CreateServerRequest): Promise<Server> {
    const formData = new FormData()
    formData.append('name', data.name)
    if (data.icon) {
      formData.append('icon', data.icon)
    }
    if (data.banner) {
      formData.append('banner', data.banner)
    }
    if (data.description) {
      formData.append('description', data.description)
    }
    if (data.isPublic !== undefined) {
      formData.append('isPublic', String(data.isPublic))
    }
    if (data.isNsfw !== undefined) {
      formData.append('isNsfw', String(data.isNsfw))
    }
    if (data.type) {
      formData.append('type', data.type)
    }
    const response = await apiClient.post<Server>(this.baseUrl, formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    })
    return response.data
  }

  async updateServer(serverId: string, data: Partial<Server>): Promise<Server> {
    const response = await apiClient.put<Server>(`${this.baseUrl}/${serverId}`, data)
    return response.data
  }

  async deleteServer(serverId: string): Promise<void> {
    await apiClient.delete(`${this.baseUrl}/${serverId}`)
  }

  async getMembers(serverId: string): Promise<ServerMember[]> {
    const response = await apiClient.get<ServerMember[]>(`${this.baseUrl}/${serverId}/members`)
    return response.data
  }

  async getRoles(serverId: string): Promise<Role[]> {
    const response = await apiClient.get<Role[]>(`${this.baseUrl}/${serverId}/roles`)
    return response.data
  }

  async createRole(serverId: string, data: Partial<Role>): Promise<Role> {
    const response = await apiClient.post<Role>(`${this.baseUrl}/${serverId}/roles`, data)
    return response.data
  }

  async updateRole(serverId: string, roleId: string, data: Partial<Role>): Promise<Role> {
    const response = await apiClient.put<Role>(`${this.baseUrl}/${serverId}/roles/${roleId}`, data)
    return response.data
  }

  async deleteRole(serverId: string, roleId: string): Promise<void> {
    await apiClient.delete(`${this.baseUrl}/${serverId}/roles/${roleId}`)
  }
}

export const serverService = ServerService.getInstance() 