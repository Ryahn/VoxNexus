const Client = require('ssh2-sftp-client');
const path = require('path');
const BaseAdapter = require('./base');
const utility = require('../../libs/utils');

/**
 * SFTP storage adapter
 * Stores files on a remote SFTP server
 */
class SFTPAdapter extends BaseAdapter {
  constructor() {
    super();

    // Validate required environment variables
    if (!process.env.SFTP_HOST || !process.env.SFTP_ROOT_PATH) {
      throw new Error('SFTP host and root path are required');
    }

    if (!process.env.SFTP_USER || (!process.env.SFTP_PASSWORD && !process.env.SFTP_PRIVATE_KEY)) {
      throw new Error('SFTP authentication credentials are required (password or private key)');
    }

    // Initialize connection pool
    this.pool = [];
    this.poolSize = parseInt(process.env.SFTP_POOL_SIZE) || 5;
    this.maxRetries = parseInt(process.env.SFTP_MAX_RETRIES) || 3;
    this.retryDelay = parseInt(process.env.SFTP_RETRY_DELAY) || 1000;
    this.connectionTimeout = parseInt(process.env.SFTP_CONNECTION_TIMEOUT) || 30000;
    this.maxFileSize = parseInt(process.env.SFTP_MAX_FILE_SIZE) || 10 * 1024 * 1024; // 10MB default

    this.config = {
      host: process.env.SFTP_HOST,
      port: process.env.SFTP_PORT || 22,
      username: process.env.SFTP_USER,
      password: process.env.SFTP_PASSWORD,
      privateKey: process.env.SFTP_PRIVATE_KEY,
      rootPath: process.env.SFTP_ROOT_PATH,
      readyTimeout: this.connectionTimeout,
      retries: this.maxRetries,
      retry_factor: 2,
      retry_minTimeout: this.retryDelay
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
    if (!clientInfo.client.isConnected()) {
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
   * Connect to SFTP with retry mechanism
   * @private
   * @param {Client} client - The SFTP client
   * @returns {Promise<void>}
   */
  async connectWithRetry(client) {
    let lastError;
    for (let i = 0; i < this.maxRetries; i++) {
      try {
        await client.connect(this.config);
        return;
      } catch (error) {
        lastError = error;
        if (i < this.maxRetries - 1) {
          await new Promise(resolve => setTimeout(resolve, this.retryDelay * Math.pow(2, i)));
        }
      }
    }
    throw new Error(`Failed to connect to SFTP after ${this.maxRetries} attempts: ${lastError.message}`);
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
      const exists = await client.exists(dirPath);
      if (!exists) {
        await client.mkdir(dirPath, true);
      }
    });
  }

  /**
   * Upload a file to SFTP
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
      await client.put(fileBuffer, fullPath);
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
      await client.put(fileBuffer, fullPath);
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
      await client.put(fileBuffer, fullPath);
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
      await client.put(fileBuffer, fullPath);
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
      await client.put(fileBuffer, fullPath);
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
      await client.put(fileBuffer, fullPath);
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
      await client.put(fileBuffer, fullPath);
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
      await client.put(fileBuffer, fullPath);
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
      const exists = await client.exists(dirPath);
      
      if (!exists) {
        return {};
      }

      const files = await client.list(dirPath);
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
      const exists = await client.exists(dirPath);
      
      if (!exists) {
        return {};
      }

      const files = await client.list(dirPath);
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
      const exists = await client.exists(dirPath);
      
      if (!exists) {
        return { attachments: [] };
      }

      const files = await client.list(dirPath);
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
      const exists = await client.exists(dirPath);
      
      if (!exists) {
        return { attachments: [] };
      }

      const files = await client.list(dirPath);
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
   * Delete a file from SFTP
   * @param {string} filePath - The path of the file to delete
   * @returns {Promise<boolean>} Whether the deletion was successful
   */
  async delete(filePath) {
    try {
      await this.executeWithRetry(async (client) => {
        const fullPath = path.join(this.config.rootPath, filePath);
        await client.delete(fullPath);
      });
      return true;
    } catch (error) {
      console.error('Error deleting file from SFTP:', error);
      return false;
    }
  }

  /**
   * Get a file from SFTP
   * @param {string} filePath - The path of the file to retrieve
   * @returns {Promise<Buffer>} The file buffer
   */
  async get(filePath) {
    return this.executeWithRetry(async (client) => {
      const fullPath = path.join(this.config.rootPath, filePath);
      return client.get(fullPath);
    });
  }
}

module.exports = SFTPAdapter;
