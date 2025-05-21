const LocalAdapter = require('./adapters/local');
const S3Adapter = require('./adapters/s3');
const FTPAdapter = require('./adapters/ftp');
const SFTPAdapter = require('./adapters/sftp');

/**
 * Service for handling file attachments
 * Uses different storage adapters based on configuration
 */
class AttachmentService {
  constructor() {
    this.adapter = this.createAdapter();
  }

  /**
   * Create the appropriate storage adapter based on configuration
   * @private
   * @returns {BaseAdapter} The configured storage adapter
   */
  createAdapter() {
    const storageType = process.env.STORAGE || 'local';

    switch (storageType.toLowerCase()) {
      case 's3':
        return new S3Adapter();
      case 'ftp':
        return new FTPAdapter();
      case 'sftp':
        return new SFTPAdapter();
      case 'local':
      default:
        return new LocalAdapter();
    }
  }

  /**
   * Upload a file attachment
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} filename - The name of the file
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path or URL where the file was stored
   */
  async uploadAttachment(fileBuffer, filename, mimeType) {
    return this.adapter.upload(fileBuffer, filename, mimeType);
  }

  /**
   * Upload a user avatar
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} userId - The user's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the avatar was stored
   */
  async uploadUserAvatar(fileBuffer, userId, mimeType) {
    return this.adapter.uploadUserAvatar(fileBuffer, userId, mimeType);
  }

  /**
   * Upload a user banner
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} userId - The user's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the banner was stored
   */
  async uploadUserBanner(fileBuffer, userId, mimeType) {
    return this.adapter.uploadUserBanner(fileBuffer, userId, mimeType);
  }

  /**
   * Upload a server icon
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} serverId - The server's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the icon was stored
   */
  async uploadServerIcon(fileBuffer, serverId, mimeType) {
    return this.adapter.uploadServerIcon(fileBuffer, serverId, mimeType);
  }

  /**
   * Upload a server banner
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} serverId - The server's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the banner was stored
   */
  async uploadServerBanner(fileBuffer, serverId, mimeType) {
    return this.adapter.uploadServerBanner(fileBuffer, serverId, mimeType);
  }

  /**
   * Upload a channel icon
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} channelId - The channel's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the icon was stored
   */
  async uploadChannelIcon(fileBuffer, channelId, mimeType) {
    return this.adapter.uploadChannelIcon(fileBuffer, channelId, mimeType);
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
    return this.adapter.uploadChannelAttachment(fileBuffer, channelId, filename, mimeType);
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
    return this.adapter.uploadGroupChatAttachment(fileBuffer, groupId, filename, mimeType);
  }

  /**
   * Get all files for a user
   * @param {string} userId - The user's snowflake ID
   * @returns {Promise<{avatar?: string, banner?: string}>} Object containing paths to user files
   */
  async getUserFiles(userId) {
    return this.adapter.getUserFiles(userId);
  }

  /**
   * Get all files for a server
   * @param {string} serverId - The server's snowflake ID
   * @returns {Promise<{icon?: string, banner?: string}>} Object containing paths to server files
   */
  async getServerFiles(serverId) {
    return this.adapter.getServerFiles(serverId);
  }

  /**
   * Get all files for a channel
   * @param {string} channelId - The channel's snowflake ID
   * @returns {Promise<{icon?: string, attachments: string[]}>} Object containing paths to channel files
   */
  async getChannelFiles(channelId) {
    return this.adapter.getChannelFiles(channelId);
  }

  /**
   * Get all files for a group chat
   * @param {string} groupId - The group chat's snowflake ID
   * @returns {Promise<{attachments: string[]}>} Object containing paths to group chat files
   */
  async getGroupChatFiles(groupId) {
    return this.adapter.getGroupChatFiles(groupId);
  }

  /**
   * Delete a file attachment
   * @param {string} filePath - The path or identifier of the file to delete
   * @returns {Promise<boolean>} Whether the deletion was successful
   */
  async deleteAttachment(filePath) {
    return this.adapter.delete(filePath);
  }

  /**
   * Get a file attachment
   * @param {string} filePath - The path or identifier of the file to retrieve
   * @returns {Promise<Buffer>} The file buffer
   */
  async getAttachment(filePath) {
    return this.adapter.get(filePath);
  }
}

module.exports = new AttachmentService();
