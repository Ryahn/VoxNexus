import { Client } from 'basic-ftp';
import path from 'path';
import BaseAdapter from './base.js';
import { utility } from '../../../libs/utils.js';

/**
 * FTP storage adapter
 * Stores files on a remote FTP server
 */
class FTPAdapter extends BaseAdapter {
  constructor() {
    super();
    this.client = new Client();
    this.host = process.env.FTP_HOST;
    this.user = process.env.FTP_USER;
    this.password = process.env.FTP_PASSWORD;
    this.port = process.env.FTP_PORT || 21;

    // Validate required environment variables
    if (!process.env.FTP_HOST || !process.env.FTP_ROOT_PATH) {
      throw new Error('FTP host and root path are required');
    }

    if (!process.env.FTP_USER || !process.env.FTP_PASSWORD) {
      throw new Error('FTP authentication credentials are required');
    }

    // Initialize connection pool
    this.pool = [];
    this.poolSize = parseInt(process.env.FTP_POOL_SIZE) || 5;
    this.maxRetries = parseInt(process.env.FTP_MAX_RETRIES) || 3;
    this.retryDelay = parseInt(process.env.FTP_RETRY_DELAY) || 1000;
    this.connectionTimeout = parseInt(process.env.FTP_CONNECTION_TIMEOUT) || 30000;
    this.maxFileSize = parseInt(process.env.FTP_MAX_FILE_SIZE) || 10 * 1024 * 1024; // 10MB default

    this.config = {
      host: process.env.FTP_HOST,
      port: process.env.FTP_PORT || 21,
      user: process.env.FTP_USER,
      password: process.env.FTP_PASSWORD,
      rootPath: process.env.FTP_ROOT_PATH,
      secure: process.env.FTP_SECURE === 'true',
      timeout: this.connectionTimeout
    };

    // Initialize the connection pool
    this.initializePool();
  }

  /**
   * Initialize the connection pool
   * @private
   */
  async initializePool() {
    for (let i = 0; i < this.poolSize; i++) {
      const client = new Client();
      this.pool.push({
        client,
        inUse: false,
        lastUsed: Date.now()
      });
    }
  }

  /**
   * Get an available client from the pool
   * @private
   * @returns {Promise<{client: Client, release: Function}>} The client and release function
   */
  async getClient() {
    // Try to find an available client
    let clientInfo = this.pool.find(c => !c.inUse);
    
    if (!clientInfo) {
      // If no client is available, wait for one to become available
      clientInfo = await new Promise(resolve => {
        const checkInterval = setInterval(() => {
          const available = this.pool.find(c => !c.inUse);
          if (available) {
            clearInterval(checkInterval);
            resolve(available);
          }
        }, 100);
      });
    }

    clientInfo.inUse = true;
    clientInfo.lastUsed = Date.now();

    // Ensure the client is connected
    if (!clientInfo.client.closed) {
      await this.connectWithRetry(clientInfo.client);
    }

    return {
      client: clientInfo.client,
      release: () => {
        clientInfo.inUse = false;
        clientInfo.lastUsed = Date.now();
      }
    };
  }

  /**
   * Connect to FTP with retry mechanism
   * @private
   * @param {Client} client - The FTP client
   * @returns {Promise<void>}
   */
  async connectWithRetry(client) {
    let lastError;
    for (let i = 0; i < this.maxRetries; i++) {
      try {
        await client.access({
          host: this.config.host,
          port: this.config.port,
          user: this.config.user,
          password: this.config.password,
          secure: this.config.secure
        });
        return;
      } catch (error) {
        lastError = error;
        if (i < this.maxRetries - 1) {
          await new Promise(resolve => setTimeout(resolve, this.retryDelay * Math.pow(2, i)));
        }
      }
    }
    throw new Error(`Failed to connect to FTP after ${this.maxRetries} attempts: ${lastError.message}`);
  }

  /**
   * Execute an operation with retry mechanism
   * @private
   * @param {Function} operation - The operation to execute
   * @returns {Promise<any>} The operation result
   */
  async executeWithRetry(operation) {
    let lastError;
    for (let i = 0; i < this.maxRetries; i++) {
      try {
        const { client, release } = await this.getClient();
        try {
          const result = await operation(client);
          release();
          return result;
        } catch (error) {
          release();
          throw error;
        }
      } catch (error) {
        lastError = error;
        if (i < this.maxRetries - 1) {
          await new Promise(resolve => setTimeout(resolve, this.retryDelay * Math.pow(2, i)));
        }
      }
    }
    throw new Error(`Operation failed after ${this.maxRetries} attempts: ${lastError.message}`);
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
   * Get the full path for a file
   * @private
   * @param {string} type - The type of attachment (user, server, channel, group)
   * @param {string} id - The snowflake ID
   * @param {string} filename - The filename
   * @returns {string} The full path
   */
  getFullPath(type, id, filename) {
    return path.join(this.config.rootPath, type, id, filename);
  }

  /**
   * Ensure a directory exists
   * @private
   * @param {string} dirPath - The directory path
   */
  async ensureDirectory(dirPath) {
    await this.executeWithRetry(async (client) => {
      try {
        await client.cd(dirPath);
      } catch (error) {
        if (error.code === 550) { // Directory not found
          await client.ensureDir(dirPath);
        } else {
          throw error;
        }
      }
    });
  }

  /**
   * Upload a file to FTP
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} filename - The name of the file
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the file was stored
   */
  async upload(fileBuffer, filename, mimeType) {
    this.validateFileSize(fileBuffer);
    
    const dirPath = path.join(this.config.rootPath, 'attachments');
    await this.ensureDirectory(dirPath);
    
    const uniqueFilename = `${utility.generateSnowflake()}${path.extname(filename)}`;
    const fullPath = path.join(dirPath, uniqueFilename);
    
    await this.executeWithRetry(async (client) => {
      await client.uploadFrom(fileBuffer, fullPath);
    });

    return `attachments/${uniqueFilename}`;
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
    
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `avatar.${ext}`;
    const dirPath = path.join(this.config.rootPath, 'user', userId);
    await this.ensureDirectory(dirPath);
    
    const fullPath = path.join(dirPath, filename);
    
    await this.executeWithRetry(async (client) => {
      await client.uploadFrom(fileBuffer, fullPath);
    });

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
    
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `banner.${ext}`;
    const dirPath = path.join(this.config.rootPath, 'user', userId);
    await this.ensureDirectory(dirPath);
    
    const fullPath = path.join(dirPath, filename);
    
    await this.executeWithRetry(async (client) => {
      await client.uploadFrom(fileBuffer, fullPath);
    });

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
    
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `icon.${ext}`;
    const dirPath = path.join(this.config.rootPath, 'server', serverId);
    await this.ensureDirectory(dirPath);
    
    const fullPath = path.join(dirPath, filename);
    
    await this.executeWithRetry(async (client) => {
      await client.uploadFrom(fileBuffer, fullPath);
    });

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
    
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `banner.${ext}`;
    const dirPath = path.join(this.config.rootPath, 'server', serverId);
    await this.ensureDirectory(dirPath);
    
    const fullPath = path.join(dirPath, filename);
    
    await this.executeWithRetry(async (client) => {
      await client.uploadFrom(fileBuffer, fullPath);
    });

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
    
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `icon.${ext}`;
    const dirPath = path.join(this.config.rootPath, 'channel', channelId);
    await this.ensureDirectory(dirPath);
    
    const fullPath = path.join(dirPath, filename);
    
    await this.executeWithRetry(async (client) => {
      await client.uploadFrom(fileBuffer, fullPath);
    });

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
    
    const ext = path.extname(filename);
    const uniqueFilename = `attachment_${utility.generateSnowflake()}${ext}`;
    const dirPath = path.join(this.config.rootPath, 'channel', channelId);
    await this.ensureDirectory(dirPath);
    
    const fullPath = path.join(dirPath, uniqueFilename);
    
    await this.executeWithRetry(async (client) => {
      await client.uploadFrom(fileBuffer, fullPath);
    });

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
    
    const ext = path.extname(filename);
    const uniqueFilename = `attachment_${utility.generateSnowflake()}${ext}`;
    const dirPath = path.join(this.config.rootPath, 'group', groupId);
    await this.ensureDirectory(dirPath);
    
    const fullPath = path.join(dirPath, uniqueFilename);
    
    await this.executeWithRetry(async (client) => {
      await client.uploadFrom(fileBuffer, fullPath);
    });

    return `group/${groupId}/${uniqueFilename}`;
  }

  /**
   * Get all files for a user
   * @param {string} userId - The user's snowflake ID
   * @returns {Promise<{avatar?: string, banner?: string}>} Object containing paths to user files
   */
  async getUserFiles(userId) {
    return this.executeWithRetry(async (client) => {
      const dirPath = path.join(this.config.rootPath, 'user', userId);
      try {
        await client.cd(dirPath);
      } catch (error) {
        if (error.code === 550) { // Directory not found
          return {};
        }
        throw error;
      }

      const files = await client.list();
      const result = {};

      for (const file of files) {
        if (file.name.startsWith('avatar.')) {
          result.avatar = `user/${userId}/${file.name}`;
        } else if (file.name.startsWith('banner.')) {
          result.banner = `user/${userId}/${file.name}`;
        }
      }

      return result;
    });
  }

  /**
   * Get all files for a server
   * @param {string} serverId - The server's snowflake ID
   * @returns {Promise<{icon?: string, banner?: string}>} Object containing paths to server files
   */
  async getServerFiles(serverId) {
    return this.executeWithRetry(async (client) => {
      const dirPath = path.join(this.config.rootPath, 'server', serverId);
      try {
        await client.cd(dirPath);
      } catch (error) {
        if (error.code === 550) { // Directory not found
          return {};
        }
        throw error;
      }

      const files = await client.list();
      const result = {};

      for (const file of files) {
        if (file.name.startsWith('icon.')) {
          result.icon = `server/${serverId}/${file.name}`;
        } else if (file.name.startsWith('banner.')) {
          result.banner = `server/${serverId}/${file.name}`;
        }
      }

      return result;
    });
  }

  /**
   * Get all files for a channel
   * @param {string} channelId - The channel's snowflake ID
   * @returns {Promise<{icon?: string, attachments: string[]}>} Object containing paths to channel files
   */
  async getChannelFiles(channelId) {
    return this.executeWithRetry(async (client) => {
      const dirPath = path.join(this.config.rootPath, 'channel', channelId);
      try {
        await client.cd(dirPath);
      } catch (error) {
        if (error.code === 550) { // Directory not found
          return { attachments: [] };
        }
        throw error;
      }

      const files = await client.list();
      const result = {
        attachments: []
      };

      for (const file of files) {
        if (file.name.startsWith('icon.')) {
          result.icon = `channel/${channelId}/${file.name}`;
        } else if (file.name.startsWith('attachment_')) {
          result.attachments.push(`channel/${channelId}/${file.name}`);
        }
      }

      return result;
    });
  }

  /**
   * Get all files for a group chat
   * @param {string} groupId - The group chat's snowflake ID
   * @returns {Promise<{attachments: string[]}>} Object containing paths to group chat files
   */
  async getGroupChatFiles(groupId) {
    return this.executeWithRetry(async (client) => {
      const dirPath = path.join(this.config.rootPath, 'group', groupId);
      try {
        await client.cd(dirPath);
      } catch (error) {
        if (error.code === 550) { // Directory not found
          return { attachments: [] };
        }
        throw error;
      }

      const files = await client.list();
      const result = {
        attachments: []
      };

      for (const file of files) {
        if (file.name.startsWith('attachment_')) {
          result.attachments.push(`group/${groupId}/${file.name}`);
        }
      }

      return result;
    });
  }

  /**
   * Delete a file from FTP
   * @param {string} filePath - The path of the file to delete
   * @returns {Promise<boolean>} Whether the deletion was successful
   */
  async delete(filePath) {
    try {
      await this.executeWithRetry(async (client) => {
        const fullPath = path.join(this.config.rootPath, filePath);
        await client.remove(fullPath);
      });
      return true;
    } catch (error) {
      console.error('Error deleting file from FTP:', error);
      return false;
    }
  }

  /**
   * Get a file from FTP
   * @param {string} filePath - The path of the file to retrieve
   * @returns {Promise<Buffer>} The file buffer
   */
  async get(filePath) {
    return this.executeWithRetry(async (client) => {
      const fullPath = path.join(this.config.rootPath, filePath);
      const chunks = [];
      await client.downloadTo(chunks, fullPath);
      return Buffer.concat(chunks);
    });
  }
}

export default FTPAdapter;
