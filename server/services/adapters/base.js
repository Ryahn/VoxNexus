/**
 * Base adapter interface for storage implementations
 * All storage adapters must implement these methods
 */
class BaseAdapter {
  /**
   * Upload a file to storage
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} filename - The name of the file
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The URL or path where the file was stored
   */
  async upload(fileBuffer, filename, mimeType) {
    throw new Error('Method not implemented');
  }

  /**
   * Delete a file from storage
   * @param {string} filePath - The path or identifier of the file to delete
   * @returns {Promise<boolean>} Whether the deletion was successful
   */
  async delete(filePath) {
    throw new Error('Method not implemented');
  }

  /**
   * Get a file from storage
   * @param {string} filePath - The path or identifier of the file to retrieve
   * @returns {Promise<Buffer>} The file buffer
   */
  async get(filePath) {
    throw new Error('Method not implemented');
  }

  /**
   * Upload a user avatar
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} userId - The user's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the avatar was stored
   */
  async uploadUserAvatar(fileBuffer, userId, mimeType) {
    throw new Error('Method not implemented');
  }

  /**
   * Upload a user banner
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} userId - The user's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the banner was stored
   */
  async uploadUserBanner(fileBuffer, userId, mimeType) {
    throw new Error('Method not implemented');
  }

  /**
   * Upload a server icon
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} serverId - The server's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the icon was stored
   */
  async uploadServerIcon(fileBuffer, serverId, mimeType) {
    throw new Error('Method not implemented');
  }

  /**
   * Upload a server banner
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} serverId - The server's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the banner was stored
   */
  async uploadServerBanner(fileBuffer, serverId, mimeType) {
    throw new Error('Method not implemented');
  }

  /**
   * Upload a channel icon
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} channelId - The channel's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the icon was stored
   */
  async uploadChannelIcon(fileBuffer, channelId, mimeType) {
    throw new Error('Method not implemented');
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
    throw new Error('Method not implemented');
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
    throw new Error('Method not implemented');
  }

  /**
   * Get all files for a user
   * @param {string} userId - The user's snowflake ID
   * @returns {Promise<{avatar?: string, banner?: string}>} Object containing paths to user files
   */
  async getUserFiles(userId) {
    throw new Error('Method not implemented');
  }

  /**
   * Get all files for a server
   * @param {string} serverId - The server's snowflake ID
   * @returns {Promise<{icon?: string, banner?: string}>} Object containing paths to server files
   */
  async getServerFiles(serverId) {
    throw new Error('Method not implemented');
  }

  /**
   * Get all files for a channel
   * @param {string} channelId - The channel's snowflake ID
   * @returns {Promise<{icon?: string, attachments: string[]}>} Object containing paths to channel files
   */
  async getChannelFiles(channelId) {
    throw new Error('Method not implemented');
  }

  /**
   * Get all files for a group chat
   * @param {string} groupId - The group chat's snowflake ID
   * @returns {Promise<{attachments: string[]}>} Object containing paths to group chat files
   */
  async getGroupChatFiles(groupId) {
    throw new Error('Method not implemented');
  }

  /**
   * Get the directory path for a specific type of attachment
   * @param {string} type - The type of attachment (user, server, channel, group)
   * @param {string} id - The snowflake ID
   * @returns {string} The directory path
   */
  getAttachmentDirectory(type, id) {
    throw new Error('Method not implemented');
  }
}

export default BaseAdapter; 