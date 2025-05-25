import { apiClient } from './client'
import type { Channel, Message, PaginatedResponse } from '@/types'

export class ChannelService {
  private static instance: ChannelService
  private readonly baseUrl = '/api/channels'

  private constructor() {}

  static getInstance(): ChannelService {
    if (!ChannelService.instance) {
      ChannelService.instance = new ChannelService()
    }
    return ChannelService.instance
  }

  async getChannels(serverId: string): Promise<Channel[]> {
    const response = await apiClient.get<Channel[]>(`${this.baseUrl}/server/${serverId}`)
    return response.data
  }

  async getChannel(channelId: string): Promise<Channel> {
    const response = await apiClient.get<Channel>(`${this.baseUrl}/${channelId}`)
    return response.data
  }

  async createChannel(serverId: string, data: Partial<Channel>): Promise<Channel> {
    const response = await apiClient.post<Channel>(`${this.baseUrl}/server/${serverId}`, data)
    return response.data
  }

  async updateChannel(channelId: string, data: Partial<Channel>): Promise<Channel> {
    const response = await apiClient.put<Channel>(`${this.baseUrl}/${channelId}`, data)
    return response.data
  }

  async deleteChannel(channelId: string): Promise<void> {
    await apiClient.delete(`${this.baseUrl}/${channelId}`)
  }

  async getMessages(channelId: string, cursor?: string): Promise<PaginatedResponse<Message>> {
    const params = cursor ? { cursor } : undefined
    const response = await apiClient.get<PaginatedResponse<Message>>(
      `${this.baseUrl}/${channelId}/messages`,
      { params }
    )
    return response.data
  }

  async createMessage(channelId: string, content: string, attachments?: File[]): Promise<Message> {
    const formData = new FormData()
    formData.append('content', content)
    if (attachments) {
      attachments.forEach(file => formData.append('attachments', file))
    }
    
    const response = await apiClient.post<Message>(
      `${this.baseUrl}/${channelId}/messages`,
      formData,
      {
        headers: {
          'Content-Type': 'multipart/form-data',
        },
      }
    )
    return response.data
  }

  async updateMessage(channelId: string, messageId: string, content: string): Promise<Message> {
    const response = await apiClient.put<Message>(
      `${this.baseUrl}/${channelId}/messages/${messageId}`,
      { content }
    )
    return response.data
  }

  async deleteMessage(channelId: string, messageId: string): Promise<void> {
    await apiClient.delete(`${this.baseUrl}/${channelId}/messages/${messageId}`)
  }
}

export const channelService = ChannelService.getInstance() 