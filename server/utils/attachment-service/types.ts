export type AttachmentType = 'user' | 'server' | 'channel' | 'group';

export interface UploadResult {
  path: string;
  url?: string;
}

export interface UserFiles {
  avatar?: string;
  banner?: string;
}
export interface ServerFiles {
  icon?: string;
  banner?: string;
}
export interface ChannelFiles {
  icon?: string;
  attachments: string[];
}
export interface GroupFiles {
  attachments: string[];
}

export interface BaseAdapter {
  upload(fileBuffer: Buffer, filename: string, mimeType: string): Promise<string>;
  delete(filePath: string): Promise<boolean>;
  get(filePath: string): Promise<Buffer>;
  uploadUserAvatar(fileBuffer: Buffer, userId: string, mimeType: string): Promise<string>;
  uploadUserBanner(fileBuffer: Buffer, userId: string, mimeType: string): Promise<string>;
  uploadServerIcon(fileBuffer: Buffer, serverId: string, mimeType: string): Promise<string>;
  uploadServerBanner(fileBuffer: Buffer, serverId: string, mimeType: string): Promise<string>;
  uploadChannelIcon(fileBuffer: Buffer, channelId: string, mimeType: string): Promise<string>;
  uploadChannelAttachment(fileBuffer: Buffer, channelId: string, filename: string, mimeType: string): Promise<string>;
  uploadGroupChatAttachment(fileBuffer: Buffer, groupId: string, filename: string, mimeType: string): Promise<string>;
  getUserFiles(userId: string): Promise<UserFiles>;
  getServerFiles(serverId: string): Promise<ServerFiles>;
  getChannelFiles(channelId: string): Promise<ChannelFiles>;
  getGroupChatFiles(groupId: string): Promise<GroupFiles>;
} 