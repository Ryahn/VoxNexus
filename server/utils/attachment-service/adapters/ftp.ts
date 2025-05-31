import { Client } from 'basic-ftp';
import type { BaseAdapter, UserFiles, ServerFiles, ChannelFiles, GroupFiles } from '../types';
import { generateSnowflake } from '../snowflake';
import path from 'path';

export default class FTPAdapter implements BaseAdapter {
  config: any;
  maxFileSize: number;
  rootPath: string;

  constructor() {
    if (!process.env.FTP_HOST || !process.env.FTP_USER || !process.env.FTP_ROOT_PATH) {
      throw new Error('Missing FTP configuration');
    }
    this.config = {
      host: process.env.FTP_HOST,
      port: parseInt(process.env.FTP_PORT || '21'),
      user: process.env.FTP_USER,
      password: process.env.FTP_PASSWORD,
      secure: process.env.FTP_SECURE === 'true',
    };
    this.maxFileSize = parseInt(process.env.FTP_MAX_FILE_SIZE || '') || 10 * 1024 * 1024;
    this.rootPath = process.env.FTP_ROOT_PATH;
  }

  private getFullPath(type: string, id: string, filename: string) {
    return path.join(this.rootPath, type, id, filename);
  }

  private async ensureDir(client: Client, dirPath: string) {
    try {
      await client.cd(dirPath);
    } catch {
      await client.ensureDir(dirPath);
    }
  }

  async upload(fileBuffer: Buffer, filename: string, mimeType: string): Promise<string> {
    if (fileBuffer.length > this.maxFileSize) throw new Error('File too large');
    const client = new Client();
    await client.access(this.config);
    const dirPath = path.join(this.rootPath, 'attachments');
    await this.ensureDir(client, dirPath);
    const uniqueFilename = `${generateSnowflake()}${path.extname(filename)}`;
    const fullPath = path.join(dirPath, uniqueFilename);
    await client.uploadFrom(Buffer.from(fileBuffer), fullPath);
    client.close();
    return `attachments/${uniqueFilename}`;
  }

  async delete(filePath: string): Promise<boolean> {
    const client = new Client();
    await client.access(this.config);
    try {
      const fullPath = path.join(this.rootPath, filePath);
      await client.remove(fullPath);
      client.close();
      return true;
    } catch {
      client.close();
      return false;
    }
  }

  async get(filePath: string): Promise<Buffer> {
    const client = new Client();
    await client.access(this.config);
    const fullPath = path.join(this.rootPath, filePath);
    const chunks: Buffer[] = [];
    await client.downloadTo(chunks, fullPath);
    client.close();
    return Buffer.concat(chunks);
  }

  async uploadUserAvatar(fileBuffer: Buffer, userId: string, mimeType: string): Promise<string> {
    if (fileBuffer.length > this.maxFileSize) throw new Error('File too large');
    const client = new Client();
    await client.access(this.config);
    const userDir = path.join(this.rootPath, 'user', userId);
    await this.ensureDir(client, userDir);
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `avatar.${ext}`;
    const filePath = path.join(userDir, filename);
    await client.uploadFrom(Buffer.from(fileBuffer), filePath);
    client.close();
    return `user/${userId}/${filename}`;
  }

  async uploadUserBanner(fileBuffer: Buffer, userId: string, mimeType: string): Promise<string> {
    if (fileBuffer.length > this.maxFileSize) throw new Error('File too large');
    const client = new Client();
    await client.access(this.config);
    const userDir = path.join(this.rootPath, 'user', userId);
    await this.ensureDir(client, userDir);
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `banner.${ext}`;
    const filePath = path.join(userDir, filename);
    await client.uploadFrom(Buffer.from(fileBuffer), filePath);
    client.close();
    return `user/${userId}/${filename}`;
  }

  async uploadServerIcon(fileBuffer: Buffer, serverId: string, mimeType: string): Promise<string> {
    if (fileBuffer.length > this.maxFileSize) throw new Error('File too large');
    const client = new Client();
    await client.access(this.config);
    const serverDir = path.join(this.rootPath, 'server', serverId);
    await this.ensureDir(client, serverDir);
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `icon.${ext}`;
    const filePath = path.join(serverDir, filename);
    await client.uploadFrom(Buffer.from(fileBuffer), filePath);
    client.close();
    return `server/${serverId}/${filename}`;
  }

  async uploadServerBanner(fileBuffer: Buffer, serverId: string, mimeType: string): Promise<string> {
    if (fileBuffer.length > this.maxFileSize) throw new Error('File too large');
    const client = new Client();
    await client.access(this.config);
    const serverDir = path.join(this.rootPath, 'server', serverId);
    await this.ensureDir(client, serverDir);
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `banner.${ext}`;
    const filePath = path.join(serverDir, filename);
    await client.uploadFrom(Buffer.from(fileBuffer), filePath);
    client.close();
    return `server/${serverId}/${filename}`;
  }

  async uploadChannelIcon(fileBuffer: Buffer, channelId: string, mimeType: string): Promise<string> {
    if (fileBuffer.length > this.maxFileSize) throw new Error('File too large');
    const client = new Client();
    await client.access(this.config);
    const channelDir = path.join(this.rootPath, 'channel', channelId);
    await this.ensureDir(client, channelDir);
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `icon.${ext}`;
    const filePath = path.join(channelDir, filename);
    await client.uploadFrom(Buffer.from(fileBuffer), filePath);
    client.close();
    return `channel/${channelId}/${filename}`;
  }

  async uploadChannelAttachment(fileBuffer: Buffer, channelId: string, filename: string, mimeType: string): Promise<string> {
    if (fileBuffer.length > this.maxFileSize) throw new Error('File too large');
    const client = new Client();
    await client.access(this.config);
    const channelDir = path.join(this.rootPath, 'channel', channelId);
    await this.ensureDir(client, channelDir);
    const ext = path.extname(filename);
    const uniqueFilename = `attachment_${generateSnowflake()}${ext}`;
    const filePath = path.join(channelDir, uniqueFilename);
    await client.uploadFrom(Buffer.from(fileBuffer), filePath);
    client.close();
    return `channel/${channelId}/${uniqueFilename}`;
  }

  async uploadGroupChatAttachment(fileBuffer: Buffer, groupId: string, filename: string, mimeType: string): Promise<string> {
    if (fileBuffer.length > this.maxFileSize) throw new Error('File too large');
    const client = new Client();
    await client.access(this.config);
    const groupDir = path.join(this.rootPath, 'group', groupId);
    await this.ensureDir(client, groupDir);
    const ext = path.extname(filename);
    const uniqueFilename = `attachment_${generateSnowflake()}${ext}`;
    const filePath = path.join(groupDir, uniqueFilename);
    await client.uploadFrom(Buffer.from(fileBuffer), filePath);
    client.close();
    return `group/${groupId}/${uniqueFilename}`;
  }

  async getUserFiles(userId: string): Promise<UserFiles> {
    const client = new Client();
    await client.access(this.config);
    const userDir = path.join(this.rootPath, 'user', userId);
    let files: string[] = [];
    try {
      files = await client.list(userDir).then(list => list.map((f: any) => f.name));
    } catch {
      client.close();
      return {};
    }
    client.close();
    const result: UserFiles = {};
    for (const file of files) {
      if (file.startsWith('avatar.')) result.avatar = `user/${userId}/${file}`;
      else if (file.startsWith('banner.')) result.banner = `user/${userId}/${file}`;
    }
    return result;
  }

  async getServerFiles(serverId: string): Promise<ServerFiles> {
    const client = new Client();
    await client.access(this.config);
    const serverDir = path.join(this.rootPath, 'server', serverId);
    let files: string[] = [];
    try {
      files = await client.list(serverDir).then(list => list.map((f: any) => f.name));
    } catch {
      client.close();
      return {};
    }
    client.close();
    const result: ServerFiles = {};
    for (const file of files) {
      if (file.startsWith('icon.')) result.icon = `server/${serverId}/${file}`;
      else if (file.startsWith('banner.')) result.banner = `server/${serverId}/${file}`;
    }
    return result;
  }

  async getChannelFiles(channelId: string): Promise<ChannelFiles> {
    const client = new Client();
    await client.access(this.config);
    const channelDir = path.join(this.rootPath, 'channel', channelId);
    let files: string[] = [];
    try {
      files = await client.list(channelDir).then(list => list.map((f: any) => f.name));
    } catch {
      client.close();
      return { attachments: [] };
    }
    client.close();
    const result: ChannelFiles = { attachments: [] };
    for (const file of files) {
      if (file.startsWith('icon.')) result.icon = `channel/${channelId}/${file}`;
      else if (file.startsWith('attachment_')) result.attachments.push(`channel/${channelId}/${file}`);
    }
    return result;
  }

  async getGroupChatFiles(groupId: string): Promise<GroupFiles> {
    const client = new Client();
    await client.access(this.config);
    const groupDir = path.join(this.rootPath, 'group', groupId);
    let files: string[] = [];
    try {
      files = await client.list(groupDir).then(list => list.map((f: any) => f.name));
    } catch {
      client.close();
      return { attachments: [] };
    }
    client.close();
    const result: GroupFiles = { attachments: [] };
    for (const file of files) {
      if (file.startsWith('attachment_')) result.attachments.push(`group/${groupId}/${file}`);
    }
    return result;
  }
} 