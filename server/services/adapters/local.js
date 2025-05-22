const fs = require('fs').promises;
const path = require('path');
const BaseAdapter = require('./base');
const utility = require('../../libs/utils');

/**
 * Local filesystem storage adapter
 * Stores files in the local filesystem
 */
class LocalAdapter extends BaseAdapter {
  constructor() {
    super();
    this.storagePath = path.join(process.cwd(), 'storage');
    this.maxFileSize = parseInt(process.env.LOCAL_MAX_FILE_SIZE) || 10 * 1024 * 1024; // 10MB default
  }

  /**
   * Validate file size
   * @private
   * @param {Buffer} fileBuffer - The file buffer to validate
   * @throws {Error} If file size exceeds the limit
   */
  validateFileSize(fileBuffer) {
    if (fileBuffer.length > this.maxFileSize) {
      throw new Error(`File size exceeds the limit of ${this.maxFileSize} bytes`);
    }
  }

  /**
   * Ensure the upload directory exists
   * @private
   */
  async ensureDirectory() {
    try {
      await fs.access(this.storagePath);
    } catch {
      await fs.mkdir(this.storagePath, { recursive: true });
    }
  }

  /**
   * Ensure a specific directory exists
   * @private
   * @param {string} dirPath - The directory path to ensure exists
   */
  async ensureDirectoryExists(dirPath) {
    try {
      await fs.access(dirPath);
    } catch {
      await fs.mkdir(dirPath, { recursive: true });
    }
  }

  /**
   * Generate a unique filename
   * @private
   * @param {string} originalFilename - The original filename
   * @returns {string} A unique filename
   */
  generateUniqueFilename(originalFilename) {
    const ext = path.extname(originalFilename);
    return `${utility.generateSnowflake()}${ext}`;
  }

  /**
   * Get the directory path for a specific type of attachment
   * @param {string} type - The type of attachment (user, server, channel, group)
   * @param {string} id - The snowflake ID
   * @returns {string} The directory path
   */
  getAttachmentDirectory(type, id) {
    return path.join(this.storagePath, type, id);
  }

  /**
   * List files in a directory
   * @private
   * @param {string} dirPath - The directory path to list files from
   * @returns {Promise<string[]>} Array of filenames
   */
  async listFiles(dirPath) {
    try {
      const files = await fs.readdir(dirPath);
      return files;
    } catch (error) {
      if (error.code === 'ENOENT') {
        return [];
      }
      throw error;
    }
  }

  /**
   * Upload a file to local storage
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} filename - The name of the file
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the file was stored
   */
  async upload(fileBuffer, filename, mimeType) {
    this.validateFileSize(fileBuffer);
    await this.ensureDirectory();
    const uniqueFilename = this.generateUniqueFilename(filename);
    const filePath = path.join(this.storagePath, uniqueFilename);
    
    await fs.writeFile(filePath, fileBuffer);
    return uniqueFilename;
  }

  /**
   * Upload a user avatar
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} userId - The user's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the avatar was stored
   */
  async uploadUserAvatar(fileBuffer, userId, mimeType) {
    this.validateFileSize(fileBuffer);
    const userDir = this.getAttachmentDirectory('user', userId);
    await this.ensureDirectoryExists(userDir);
    
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `avatar.${ext}`;
    const filePath = path.join(userDir, filename);
    
    await fs.writeFile(filePath, fileBuffer);
    return `user/${userId}/${filename}`;
  }

  /**
   * Upload a user banner
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} userId - The user's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the banner was stored
   */
  async uploadUserBanner(fileBuffer, userId, mimeType) {
    this.validateFileSize(fileBuffer);
    const userDir = this.getAttachmentDirectory('user', userId);
    await this.ensureDirectoryExists(userDir);
    
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `banner.${ext}`;
    const filePath = path.join(userDir, filename);
    
    await fs.writeFile(filePath, fileBuffer);
    return `user/${userId}/${filename}`;
  }

  /**
   * Upload a server icon
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} serverId - The server's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the icon was stored
   */
  async uploadServerIcon(fileBuffer, serverId, mimeType) {
    this.validateFileSize(fileBuffer);
    const serverDir = this.getAttachmentDirectory('server', serverId);
    await this.ensureDirectoryExists(serverDir);
    
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `icon.${ext}`;
    const filePath = path.join(serverDir, filename);
    
    await fs.writeFile(filePath, fileBuffer);
    return `server/${serverId}/${filename}`;
  }

  /**
   * Upload a server banner
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} serverId - The server's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the banner was stored
   */
  async uploadServerBanner(fileBuffer, serverId, mimeType) {
    this.validateFileSize(fileBuffer);
    const serverDir = this.getAttachmentDirectory('server', serverId);
    await this.ensureDirectoryExists(serverDir);
    
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `banner.${ext}`;
    const filePath = path.join(serverDir, filename);
    
    await fs.writeFile(filePath, fileBuffer);
    return `server/${serverId}/${filename}`;
  }

  /**
   * Upload a channel icon
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} channelId - The channel's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the icon was stored
   */
  async uploadChannelIcon(fileBuffer, channelId, mimeType) {
    this.validateFileSize(fileBuffer);
    const channelDir = this.getAttachmentDirectory('channel', channelId);
    await this.ensureDirectoryExists(channelDir);
    
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `icon.${ext}`;
    const filePath = path.join(channelDir, filename);
    
    await fs.writeFile(filePath, fileBuffer);
    return `channel/${channelId}/${filename}`;
  }

  /**
   * Upload a channel attachment
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} channelId - The channel's snowflake ID
   * @param {string} filename - The original filename
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the attachment was stored
   */
  async uploadChannelAttachment(fileBuffer, channelId, filename, mimeType) {
    this.validateFileSize(fileBuffer);
    const channelDir = this.getAttachmentDirectory('channel', channelId);
    await this.ensureDirectoryExists(channelDir);
    
    const ext = path.extname(filename);
    const uniqueFilename = `attachment_${utility.generateSnowflake()}${ext}`;
    const filePath = path.join(channelDir, uniqueFilename);
    
    await fs.writeFile(filePath, fileBuffer);
    return `channel/${channelId}/${uniqueFilename}`;
  }

  /**
   * Upload a group chat attachment
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} groupId - The group chat's snowflake ID
   * @param {string} filename - The original filename
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the attachment was stored
   */
  async uploadGroupChatAttachment(fileBuffer, groupId, filename, mimeType) {
    this.validateFileSize(fileBuffer);
    const groupDir = this.getAttachmentDirectory('group', groupId);
    await this.ensureDirectoryExists(groupDir);
    
    const ext = path.extname(filename);
    const uniqueFilename = `attachment_${utility.generateSnowflake()}${ext}`;
    const filePath = path.join(groupDir, uniqueFilename);
    
    await fs.writeFile(filePath, fileBuffer);
    return `group/${groupId}/${uniqueFilename}`;
  }

  /**
   * Get all files for a user
   * @param {string} userId - The user's snowflake ID
   * @returns {Promise<{avatar?: string, banner?: string}>} Object containing paths to user files
   */
  async getUserFiles(userId) {
    const userDir = this.getAttachmentDirectory('user', userId);
    const files = await this.listFiles(userDir);
    
    const result = {};
    for (const file of files) {
      if (file.startsWith('avatar.')) {
        result.avatar = `user/${userId}/${file}`;
      } else if (file.startsWith('banner.')) {
        result.banner = `user/${userId}/${file}`;
      }
    }
    
    return result;
  }

  /**
   * Get all files for a server
   * @param {string} serverId - The server's snowflake ID
   * @returns {Promise<{icon?: string, banner?: string}>} Object containing paths to server files
   */
  async getServerFiles(serverId) {
    const serverDir = this.getAttachmentDirectory('server', serverId);
    const files = await this.listFiles(serverDir);
    
    const result = {};
    for (const file of files) {
      if (file.startsWith('icon.')) {
        result.icon = `server/${serverId}/${file}`;
      } else if (file.startsWith('banner.')) {
        result.banner = `server/${serverId}/${file}`;
      }
    }
    
    return result;
  }

  /**
   * Get all files for a channel
   * @param {string} channelId - The channel's snowflake ID
   * @returns {Promise<{icon?: string, attachments: string[]}>} Object containing paths to channel files
   */
  async getChannelFiles(channelId) {
    const channelDir = this.getAttachmentDirectory('channel', channelId);
    const files = await this.listFiles(channelDir);
    
    const result = {
      attachments: []
    };
    
    for (const file of files) {
      if (file.startsWith('icon.')) {
        result.icon = `channel/${channelId}/${file}`;
      } else if (file.startsWith('attachment_')) {
        result.attachments.push(`channel/${channelId}/${file}`);
      }
    }
    
    return result;
  }

  /**
   * Get all files for a group chat
   * @param {string} groupId - The group chat's snowflake ID
   * @returns {Promise<{attachments: string[]}>} Object containing paths to group chat files
   */
  async getGroupChatFiles(groupId) {
    const groupDir = this.getAttachmentDirectory('group', groupId);
    const files = await this.listFiles(groupDir);
    
    const result = {
      attachments: []
    };
    
    for (const file of files) {
      if (file.startsWith('attachment_')) {
        result.attachments.push(`group/${groupId}/${file}`);
      }
    }
    
    return result;
  }

  /**
   * Delete a file from local storage
   * @param {string} filePath - The path of the file to delete
   * @returns {Promise<boolean>} Whether the deletion was successful
   */
  async delete(filePath) {
    try {
      const fullPath = path.join(this.storagePath, filePath);
      await fs.unlink(fullPath);
      return true;
    } catch (error) {
      console.error('Error deleting file:', error);
      return false;
    }
  }

  /**
   * Get a file from local storage
   * @param {string} filePath - The path of the file to retrieve
   * @returns {Promise<Buffer>} The file buffer
   */
  async get(filePath) {
    const fullPath = path.join(this.storagePath, filePath);
    return fs.readFile(fullPath);
  }
}

module.exports = LocalAdapter;
