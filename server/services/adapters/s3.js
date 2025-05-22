const { S3Client, PutObjectCommand, DeleteObjectCommand, GetObjectCommand, ListObjectsV2Command } = require('@aws-sdk/client-s3');
const { getSignedUrl } = require('@aws-sdk/s3-request-presigner');
const BaseAdapter = require('./base');
const utility = require('../../libs/utils');

/**
 * S3 storage adapter
 * Stores files in AWS S3
 */
class S3Adapter extends BaseAdapter {
  constructor() {
    super();

    if (!process.env.AWS_REGION || !process.env.AWS_ACCESS_KEY_ID || !process.env.AWS_SECRET_ACCESS_KEY || !process.env.AWS_S3_BUCKET) {
      throw new Error('AWS credentials are not set');
    }

    this.client = new S3Client({
      region: process.env.AWS_REGION,
      credentials: {
        accessKeyId: process.env.AWS_ACCESS_KEY_ID,
        secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY
      }
    });
    this.bucket = process.env.AWS_S3_BUCKET;
  }

  /**
   * Get the S3 key for a file
   * @private
   * @param {string} type - The type of attachment (user, server, channel, group)
   * @param {string} id - The snowflake ID
   * @param {string} filename - The filename
   * @returns {string} The S3 key
   */
  getS3Key(type, id, filename) {
    return `${type}/${id}/${filename}`;
  }

  /**
   * Upload a file to S3
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} filename - The name of the file
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the file was stored
   */
  async upload(fileBuffer, filename, mimeType) {
    const key = this.getS3Key('attachments', utility.generateSnowflake(), filename);
    
    await this.client.send(new PutObjectCommand({
      Bucket: this.bucket,
      Key: key,
      Body: fileBuffer,
      ContentType: mimeType
    }));

    return key;
  }

  /**
   * Upload a user avatar
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} userId - The user's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the avatar was stored
   */
  async uploadUserAvatar(fileBuffer, userId, mimeType) {
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `avatar.${ext}`;
    const key = this.getS3Key('user', userId, filename);
    
    await this.client.send(new PutObjectCommand({
      Bucket: this.bucket,
      Key: key,
      Body: fileBuffer,
      ContentType: mimeType
    }));

    return key;
  }

  /**
   * Upload a user banner
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} userId - The user's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the banner was stored
   */
  async uploadUserBanner(fileBuffer, userId, mimeType) {
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `banner.${ext}`;
    const key = this.getS3Key('user', userId, filename);
    
    await this.client.send(new PutObjectCommand({
      Bucket: this.bucket,
      Key: key,
      Body: fileBuffer,
      ContentType: mimeType
    }));

    return key;
  }

  /**
   * Upload a server icon
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} serverId - The server's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the icon was stored
   */
  async uploadServerIcon(fileBuffer, serverId, mimeType) {
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `icon.${ext}`;
    const key = this.getS3Key('server', serverId, filename);
    
    await this.client.send(new PutObjectCommand({
      Bucket: this.bucket,
      Key: key,
      Body: fileBuffer,
      ContentType: mimeType
    }));

    return key;
  }

  /**
   * Upload a server banner
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} serverId - The server's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the banner was stored
   */
  async uploadServerBanner(fileBuffer, serverId, mimeType) {
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `banner.${ext}`;
    const key = this.getS3Key('server', serverId, filename);
    
    await this.client.send(new PutObjectCommand({
      Bucket: this.bucket,
      Key: key,
      Body: fileBuffer,
      ContentType: mimeType
    }));

    return key;
  }

  /**
   * Upload a channel icon
   * @param {Buffer} fileBuffer - The file buffer to upload
   * @param {string} channelId - The channel's snowflake ID
   * @param {string} mimeType - The MIME type of the file
   * @returns {Promise<string>} The path where the icon was stored
   */
  async uploadChannelIcon(fileBuffer, channelId, mimeType) {
    const ext = mimeType.split('/')[1] || 'png';
    const filename = `icon.${ext}`;
    const key = this.getS3Key('channel', channelId, filename);
    
    await this.client.send(new PutObjectCommand({
      Bucket: this.bucket,
      Key: key,
      Body: fileBuffer,
      ContentType: mimeType
    }));

    return key;
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
    const ext = filename.split('.').pop();
    const uniqueFilename = `attachment_${utility.generateSnowflake()}.${ext}`;
    const key = this.getS3Key('channel', channelId, uniqueFilename);
    
    await this.client.send(new PutObjectCommand({
      Bucket: this.bucket,
      Key: key,
      Body: fileBuffer,
      ContentType: mimeType
    }));

    return key;
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
    const ext = filename.split('.').pop();
    const uniqueFilename = `attachment_${utility.generateSnowflake()}.${ext}`;
    const key = this.getS3Key('group', groupId, uniqueFilename);
    
    await this.client.send(new PutObjectCommand({
      Bucket: this.bucket,
      Key: key,
      Body: fileBuffer,
      ContentType: mimeType
    }));

    return key;
  }

  /**
   * Get all files for a user
   * @param {string} userId - The user's snowflake ID
   * @returns {Promise<{avatar?: string, banner?: string}>} Object containing paths to user files
   */
  async getUserFiles(userId) {
    const prefix = `user/${userId}/`;
    const response = await this.client.send(new ListObjectsV2Command({
      Bucket: this.bucket,
      Prefix: prefix
    }));

    const result = {};
    for (const object of response.Contents || []) {
      const filename = object.Key.split('/').pop();
      if (filename.startsWith('avatar.')) {
        result.avatar = object.Key;
      } else if (filename.startsWith('banner.')) {
        result.banner = object.Key;
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
    const prefix = `server/${serverId}/`;
    const response = await this.client.send(new ListObjectsV2Command({
      Bucket: this.bucket,
      Prefix: prefix
    }));

    const result = {};
    for (const object of response.Contents || []) {
      const filename = object.Key.split('/').pop();
      if (filename.startsWith('icon.')) {
        result.icon = object.Key;
      } else if (filename.startsWith('banner.')) {
        result.banner = object.Key;
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
    const prefix = `channel/${channelId}/`;
    const response = await this.client.send(new ListObjectsV2Command({
      Bucket: this.bucket,
      Prefix: prefix
    }));

    const result = {
      attachments: []
    };

    for (const object of response.Contents || []) {
      const filename = object.Key.split('/').pop();
      if (filename.startsWith('icon.')) {
        result.icon = object.Key;
      } else if (filename.startsWith('attachment_')) {
        result.attachments.push(object.Key);
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
    const prefix = `group/${groupId}/`;
    const response = await this.client.send(new ListObjectsV2Command({
      Bucket: this.bucket,
      Prefix: prefix
    }));

    const result = {
      attachments: []
    };

    for (const object of response.Contents || []) {
      const filename = object.Key.split('/').pop();
      if (filename.startsWith('attachment_')) {
        result.attachments.push(object.Key);
      }
    }

    return result;
  }

  /**
   * Delete a file from S3
   * @param {string} filePath - The path of the file to delete
   * @returns {Promise<boolean>} Whether the deletion was successful
   */
  async delete(filePath) {
    try {
      await this.client.send(new DeleteObjectCommand({
        Bucket: this.bucket,
        Key: filePath
      }));
      return true;
    } catch (error) {
      console.error('Error deleting file from S3:', error);
      return false;
    }
  }

  /**
   * Get a file from S3
   * @param {string} filePath - The path of the file to retrieve
   * @returns {Promise<Buffer>} The file buffer
   */
  async get(filePath) {
    const response = await this.client.send(new GetObjectCommand({
      Bucket: this.bucket,
      Key: filePath
    }));

    return Buffer.from(await response.Body.transformToByteArray());
  }
}

module.exports = S3Adapter;
