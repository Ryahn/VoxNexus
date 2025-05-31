import type { BaseAdapter as IBaseAdapter, UserFiles, ServerFiles, ChannelFiles, GroupFiles } from '../types'

export class BaseAdapter implements IBaseAdapter {
  async upload(fileBuffer: Buffer, filename: string, mimeType: string): Promise<string> {
    throw new Error('Method not implemented')
  }
  async delete(filePath: string): Promise<boolean> {
    throw new Error('Method not implemented')
  }
  async get(filePath: string): Promise<Buffer> {
    throw new Error('Method not implemented')
  }
  async uploadUserAvatar(fileBuffer: Buffer, userId: string, mimeType: string): Promise<string> {
    throw new Error('Method not implemented')
  }
  async uploadUserBanner(fileBuffer: Buffer, userId: string, mimeType: string): Promise<string> {
    throw new Error('Method not implemented')
  }
  async uploadServerIcon(fileBuffer: Buffer, serverId: string, mimeType: string): Promise<string> {
    throw new Error('Method not implemented')
  }
  async uploadServerBanner(fileBuffer: Buffer, serverId: string, mimeType: string): Promise<string> {
    throw new Error('Method not implemented')
  }
  async uploadChannelIcon(fileBuffer: Buffer, channelId: string, mimeType: string): Promise<string> {
    throw new Error('Method not implemented')
  }
  async uploadChannelAttachment(fileBuffer: Buffer, channelId: string, filename: string, mimeType: string): Promise<string> {
    throw new Error('Method not implemented')
  }
  async uploadGroupChatAttachment(fileBuffer: Buffer, groupId: string, filename: string, mimeType: string): Promise<string> {
    throw new Error('Method not implemented')
  }
  async getUserFiles(userId: string): Promise<UserFiles> {
    throw new Error('Method not implemented')
  }
  async getServerFiles(serverId: string): Promise<ServerFiles> {
    throw new Error('Method not implemented')
  }
  async getChannelFiles(channelId: string): Promise<ChannelFiles> {
    throw new Error('Method not implemented')
  }
  async getGroupChatFiles(groupId: string): Promise<GroupFiles> {
    throw new Error('Method not implemented')
  }
}

export default BaseAdapter; 