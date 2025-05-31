import fs from 'fs/promises'
import path from 'path'
import { BaseAdapter } from './base'
import type { UserFiles, ServerFiles, ChannelFiles, GroupFiles } from '../types'
import { generateSnowflake } from '../snowflake'

export default class LocalAdapter extends BaseAdapter {
  storagePath: string
  maxFileSize: number

  constructor() {
    super()
    this.storagePath = path.join(process.cwd(), 'storage')
    this.maxFileSize = parseInt(process.env.LOCAL_MAX_FILE_SIZE || '') || 10 * 1024 * 1024
  }

  private async ensureDirectoryExists(dirPath: string) {
    try {
      await fs.access(dirPath)
    } catch {
      await fs.mkdir(dirPath, { recursive: true })
    }
  }

  private generateUniqueFilename(originalFilename: string): string {
    const ext = path.extname(originalFilename)
    return `${generateSnowflake()}${ext}`
  }

  private getAttachmentDirectory(type: string, id: string): string {
    return path.join(this.storagePath, type, id)
  }

  override async upload(fileBuffer: Buffer, filename: string, mimeType: string): Promise<string> {
    if (fileBuffer.length > this.maxFileSize) throw new Error('File too large')
    await this.ensureDirectoryExists(this.storagePath)
    const uniqueFilename = this.generateUniqueFilename(filename)
    const filePath = path.join(this.storagePath, uniqueFilename)
    await fs.writeFile(filePath, fileBuffer)
    return uniqueFilename
  }

  override async delete(filePath: string): Promise<boolean> {
    try {
      const fullPath = path.join(this.storagePath, filePath)
      await fs.unlink(fullPath)
      return true
    } catch {
      return false
    }
  }

  override async get(filePath: string): Promise<Buffer> {
    const fullPath = path.join(this.storagePath, filePath)
    return fs.readFile(fullPath)
  }

  override async uploadUserAvatar(fileBuffer: Buffer, userId: string, mimeType: string): Promise<string> {
    if (fileBuffer.length > this.maxFileSize) throw new Error('File too large')
    const userDir = this.getAttachmentDirectory('user', userId)
    await this.ensureDirectoryExists(userDir)
    const ext = mimeType.split('/')[1] || 'png'
    const filename = `avatar.${ext}`
    const filePath = path.join(userDir, filename)
    await fs.writeFile(filePath, fileBuffer)
    return `user/${userId}/${filename}`
  }

  override async uploadUserBanner(fileBuffer: Buffer, userId: string, mimeType: string): Promise<string> {
    if (fileBuffer.length > this.maxFileSize) throw new Error('File too large')
    const userDir = this.getAttachmentDirectory('user', userId)
    await this.ensureDirectoryExists(userDir)
    const ext = mimeType.split('/')[1] || 'png'
    const filename = `banner.${ext}`
    const filePath = path.join(userDir, filename)
    await fs.writeFile(filePath, fileBuffer)
    return `user/${userId}/${filename}`
  }

  override async uploadServerIcon(fileBuffer: Buffer, serverId: string, mimeType: string): Promise<string> {
    if (fileBuffer.length > this.maxFileSize) throw new Error('File too large')
    const serverDir = this.getAttachmentDirectory('server', serverId)
    await this.ensureDirectoryExists(serverDir)
    const ext = mimeType.split('/')[1] || 'png'
    const filename = `icon.${ext}`
    const filePath = path.join(serverDir, filename)
    await fs.writeFile(filePath, fileBuffer)
    return `server/${serverId}/${filename}`
  }

  override async uploadServerBanner(fileBuffer: Buffer, serverId: string, mimeType: string): Promise<string> {
    if (fileBuffer.length > this.maxFileSize) throw new Error('File too large')
    const serverDir = this.getAttachmentDirectory('server', serverId)
    await this.ensureDirectoryExists(serverDir)
    const ext = mimeType.split('/')[1] || 'png'
    const filename = `banner.${ext}`
    const filePath = path.join(serverDir, filename)
    await fs.writeFile(filePath, fileBuffer)
    return `server/${serverId}/${filename}`
  }

  override async uploadChannelIcon(fileBuffer: Buffer, channelId: string, mimeType: string): Promise<string> {
    if (fileBuffer.length > this.maxFileSize) throw new Error('File too large')
    const channelDir = this.getAttachmentDirectory('channel', channelId)
    await this.ensureDirectoryExists(channelDir)
    const ext = mimeType.split('/')[1] || 'png'
    const filename = `icon.${ext}`
    const filePath = path.join(channelDir, filename)
    await fs.writeFile(filePath, fileBuffer)
    return `channel/${channelId}/${filename}`
  }

  override async uploadChannelAttachment(fileBuffer: Buffer, channelId: string, filename: string, mimeType: string): Promise<string> {
    if (fileBuffer.length > this.maxFileSize) throw new Error('File too large')
    const channelDir = this.getAttachmentDirectory('channel', channelId)
    await this.ensureDirectoryExists(channelDir)
    const ext = path.extname(filename)
    const uniqueFilename = `attachment_${generateSnowflake()}${ext}`
    const filePath = path.join(channelDir, uniqueFilename)
    await fs.writeFile(filePath, fileBuffer)
    return `channel/${channelId}/${uniqueFilename}`
  }

  override async uploadGroupChatAttachment(fileBuffer: Buffer, groupId: string, filename: string, mimeType: string): Promise<string> {
    if (fileBuffer.length > this.maxFileSize) throw new Error('File too large')
    const groupDir = this.getAttachmentDirectory('group', groupId)
    await this.ensureDirectoryExists(groupDir)
    const ext = path.extname(filename)
    const uniqueFilename = `attachment_${generateSnowflake()}${ext}`
    const filePath = path.join(groupDir, uniqueFilename)
    await fs.writeFile(filePath, fileBuffer)
    return `group/${groupId}/${uniqueFilename}`
  }

  override async getUserFiles(userId: string): Promise<UserFiles> {
    const userDir = this.getAttachmentDirectory('user', userId)
    let files: string[] = []
    try {
      files = await fs.readdir(userDir)
    } catch {
      return {}
    }
    const result: UserFiles = {}
    for (const file of files) {
      if (file.startsWith('avatar.')) result.avatar = `user/${userId}/${file}`
      else if (file.startsWith('banner.')) result.banner = `user/${userId}/${file}`
    }
    return result
  }

  override async getServerFiles(serverId: string): Promise<ServerFiles> {
    const serverDir = this.getAttachmentDirectory('server', serverId)
    let files: string[] = []
    try {
      files = await fs.readdir(serverDir)
    } catch {
      return {}
    }
    const result: ServerFiles = {}
    for (const file of files) {
      if (file.startsWith('icon.')) result.icon = `server/${serverId}/${file}`
      else if (file.startsWith('banner.')) result.banner = `server/${serverId}/${file}`
    }
    return result
  }

  override async getChannelFiles(channelId: string): Promise<ChannelFiles> {
    const channelDir = this.getAttachmentDirectory('channel', channelId)
    let files: string[] = []
    try {
      files = await fs.readdir(channelDir)
    } catch {
      return { attachments: [] }
    }
    const result: ChannelFiles = { attachments: [] }
    for (const file of files) {
      if (file.startsWith('icon.')) result.icon = `channel/${channelId}/${file}`
      else if (file.startsWith('attachment_')) result.attachments.push(`channel/${channelId}/${file}`)
    }
    return result
  }

  override async getGroupChatFiles(groupId: string): Promise<GroupFiles> {
    const groupDir = this.getAttachmentDirectory('group', groupId)
    let files: string[] = []
    try {
      files = await fs.readdir(groupDir)
    } catch {
      return { attachments: [] }
    }
    const result: GroupFiles = { attachments: [] }
    for (const file of files) {
      if (file.startsWith('attachment_')) result.attachments.push(`group/${groupId}/${file}`)
    }
    return result
  }
} 