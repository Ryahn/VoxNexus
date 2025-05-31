import { S3Client, PutObjectCommand, DeleteObjectCommand, GetObjectCommand, ListObjectsV2Command } from '@aws-sdk/client-s3';
import type { BaseAdapter, UserFiles, ServerFiles, ChannelFiles, GroupFiles } from '../types';
import { generateSnowflake } from '../snowflake';
import path from 'path';

export default class S3Adapter implements BaseAdapter {
  client: S3Client;
  bucket: string;

  constructor() {
    if (!process.env.AWS_REGION || !process.env.AWS_ACCESS_KEY_ID || !process.env.AWS_SECRET_ACCESS_KEY || !process.env.AWS_S3_BUCKET) {
      throw new Error('Missing AWS S3 configuration');
    }
    this.client = new S3Client({
      region: process.env.AWS_REGION,
      credentials: {
        accessKeyId: process.env.AWS_ACCESS_KEY_ID,
        secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
      },
    });
    this.bucket = process.env.AWS_S3_BUCKET;
  }

  private getS3Key(type: string, id: string, filename: string) {
    return `${type}/${id}/${filename}`;
  }

  async upload(fileBuffer: Buffer, filename: string, mimeType: string): Promise<string> {
    const key = this.getS3Key('attachments', generateSnowflake(), filename);
    await this.client.send(new PutObjectCommand({
      Bucket: this.bucket,
      Key: key,
      Body: fileBuffer,
      ContentType: mimeType,
    }));
    return key;
  }

  async delete(filePath: string): Promise<boolean> {
    try {
      await this.client.send(new DeleteObjectCommand({ Bucket: this.bucket, Key: filePath }));
      return true;
    } catch {
      return false;
    }
  }

  async get(filePath: string): Promise<Buffer> {
    const res = await this.client.send(new GetObjectCommand({ Bucket: this.bucket, Key: filePath }));
    // @ts-ignore
    return Buffer.from(await res.Body.transformToByteArray());
  }

  async uploadUserAvatar(fileBuffer: Buffer, userId: string, mimeType: string): Promise<string> {
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `avatar.${ext}`;
    const key = this.getS3Key('user', userId, filename);
    await this.client.send(new PutObjectCommand({ Bucket: this.bucket, Key: key, Body: fileBuffer, ContentType: mimeType }));
    return key;
  }

  async uploadUserBanner(fileBuffer: Buffer, userId: string, mimeType: string): Promise<string> {
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `banner.${ext}`;
    const key = this.getS3Key('user', userId, filename);
    await this.client.send(new PutObjectCommand({ Bucket: this.bucket, Key: key, Body: fileBuffer, ContentType: mimeType }));
    return key;
  }

  async uploadServerIcon(fileBuffer: Buffer, serverId: string, mimeType: string): Promise<string> {
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `icon.${ext}`;
    const key = this.getS3Key('server', serverId, filename);
    await this.client.send(new PutObjectCommand({ Bucket: this.bucket, Key: key, Body: fileBuffer, ContentType: mimeType }));
    return key;
  }

  async uploadServerBanner(fileBuffer: Buffer, serverId: string, mimeType: string): Promise<string> {
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `banner.${ext}`;
    const key = this.getS3Key('server', serverId, filename);
    await this.client.send(new PutObjectCommand({ Bucket: this.bucket, Key: key, Body: fileBuffer, ContentType: mimeType }));
    return key;
  }

  async uploadChannelIcon(fileBuffer: Buffer, channelId: string, mimeType: string): Promise<string> {
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `icon.${ext}`;
    const key = this.getS3Key('channel', channelId, filename);
    await this.client.send(new PutObjectCommand({ Bucket: this.bucket, Key: key, Body: fileBuffer, ContentType: mimeType }));
    return key;
  }

  async uploadChannelAttachment(fileBuffer: Buffer, channelId: string, filename: string, mimeType: string): Promise<string> {
    const ext = path.extname(filename);
    const uniqueFilename = `attachment_${generateSnowflake()}${ext}`;
    const key = this.getS3Key('channel', channelId, uniqueFilename);
    await this.client.send(new PutObjectCommand({ Bucket: this.bucket, Key: key, Body: fileBuffer, ContentType: mimeType }));
    return key;
  }

  async uploadGroupChatAttachment(fileBuffer: Buffer, groupId: string, filename: string, mimeType: string): Promise<string> {
    const ext = path.extname(filename);
    const uniqueFilename = `attachment_${generateSnowflake()}${ext}`;
    const key = this.getS3Key('group', groupId, uniqueFilename);
    await this.client.send(new PutObjectCommand({ Bucket: this.bucket, Key: key, Body: fileBuffer, ContentType: mimeType }));
    return key;
  }

  async getUserFiles(userId: string): Promise<UserFiles> {
    const prefix = `user/${userId}/`;
    const res = await this.client.send(new ListObjectsV2Command({ Bucket: this.bucket, Prefix: prefix }));
    const result: UserFiles = {};
    for (const obj of res.Contents || []) {
      const filename = obj.Key!.split('/').pop()!;
      if (filename.startsWith('avatar.')) result.avatar = obj.Key!;
      else if (filename.startsWith('banner.')) result.banner = obj.Key!;
    }
    return result;
  }

  async getServerFiles(serverId: string): Promise<ServerFiles> {
    const prefix = `server/${serverId}/`;
    const res = await this.client.send(new ListObjectsV2Command({ Bucket: this.bucket, Prefix: prefix }));
    const result: ServerFiles = {};
    for (const obj of res.Contents || []) {
      const filename = obj.Key!.split('/').pop()!;
      if (filename.startsWith('icon.')) result.icon = obj.Key!;
      else if (filename.startsWith('banner.')) result.banner = obj.Key!;
    }
    return result;
  }

  async getChannelFiles(channelId: string): Promise<ChannelFiles> {
    const prefix = `channel/${channelId}/`;
    const res = await this.client.send(new ListObjectsV2Command({ Bucket: this.bucket, Prefix: prefix }));
    const result: ChannelFiles = { attachments: [] };
    for (const obj of res.Contents || []) {
      const filename = obj.Key!.split('/').pop()!;
      if (filename.startsWith('icon.')) result.icon = obj.Key!;
      else if (filename.startsWith('attachment_')) result.attachments.push(obj.Key!);
    }
    return result;
  }

  async getGroupChatFiles(groupId: string): Promise<GroupFiles> {
    const prefix = `group/${groupId}/`;
    const res = await this.client.send(new ListObjectsV2Command({ Bucket: this.bucket, Prefix: prefix }));
    const result: GroupFiles = { attachments: [] };
    for (const obj of res.Contents || []) {
      const filename = obj.Key!.split('/').pop()!;
      if (filename.startsWith('attachment_')) result.attachments.push(obj.Key!);
    }
    return result;
  }
} 