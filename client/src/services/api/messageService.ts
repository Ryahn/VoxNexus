import { apiClient } from './client'
import type { Message, CreateMessageRequest, MessagesResponse } from './types'

export class MessageService {
  private static instance: MessageService
  private readonly baseUrl = '/api/messages'

  private constructor() {}

  static getInstance(): MessageService {
    if (!MessageService.instance) {
      MessageService.instance = new MessageService()
    }
    return MessageService.instance
  }

  async getMessages(channelId: string, cursor?: string): Promise<MessagesResponse> {
    const url = `${this.baseUrl}/channel/${channelId}`
    const params = cursor ? { cursor } : undefined
    const response = await apiClient.get<MessagesResponse>(url, { params })
    return response.data
  }

  async createMessage(request: CreateMessageRequest): Promise<Message> {
    const response = await apiClient.post<Message>(this.baseUrl, request)
    return response.data
  }

  async deleteMessage(messageId: string): Promise<void> {
    await apiClient.delete(`${this.baseUrl}/${messageId}`)
  }

  async updateMessage(messageId: string, content: string): Promise<Message> {
    const response = await apiClient.put<Message>(`${this.baseUrl}/${messageId}`, { content })
    return response.data
  }
}

export const messageService = MessageService.getInstance() 