import mongoose from 'mongoose';
import utility from '../../utils';

const AttachmentSchema = new mongoose.Schema({
  _id: {
    type: String,
    default: () => utility.generateSnowflake().toString()
  },
  url: {
    type: String,
    required: true,
    validate: {
      validator: function(v) {
        return !v || /^(https?:\/\/|\/)[^\s]+$/.test(v);
      },
      message: 'URL must be a valid URL or file path'
    }
  },
  type: {
    type: String,
    enum: ['image', 'video', 'audio', 'file'],
    default: 'file'
  },
  size: {
    type: Number,
    required: true
  },
  conversationType: {
    type: String,
    enum: ['server', 'dm', 'group'],
    required: true
  },
  channel: {
    type: String,
    ref: 'Channel'
  },
  server: {
    type: String,
    ref: 'Server'
  },
  createdAt: {
    type: Date,
    default: Date.now
  },
  updatedAt: {
    type: Date,
    default: Date.now
  }
}, {
  timestamps: true
});

// Add validation to ensure proper fields are set based on conversationType
AttachmentSchema.pre('save', function(next) {
  if (this.conversationType === 'server') {
    if (!this.channel || !this.server) {
      next(new Error('Channel and server are required for server attachments'));
    }
  } else if (this.conversationType === 'dm' || this.conversationType === 'group') {
    if (this.server) {
      next(new Error('Server should not be set for DM or group attachments'));
    }
  }
  next();
});

AttachmentSchema.index({ type: 1, size: 1 });
AttachmentSchema.index({ url: 1, server: 1 });
AttachmentSchema.index({ createdAt: 1 });
AttachmentSchema.index({ conversationType: 1 });

// Method to update attachment metadata
AttachmentSchema.methods.updateMetadata = async function(metadata) {
  const allowedUpdates = ['type', 'size'];
  Object.keys(metadata).forEach(key => {
    if (allowedUpdates.includes(key)) {
      this[key] = metadata[key];
    }
  });
  await this.save();
};

// Method to update URL
AttachmentSchema.methods.updateUrl = async function(newUrl) {
  if (/^(https?:\/\/|\/)[^\s]+$/.test(newUrl)) {
    this.url = newUrl;
    await this.save();
  }
};

// Method to validate file type
AttachmentSchema.methods.validateFileType = function() {
  const imageTypes = ['image/jpeg', 'image/png', 'image/gif', 'image/webp'];
  const videoTypes = ['video/mp4', 'video/webm', 'video/ogg'];
  const audioTypes = ['audio/mpeg', 'audio/ogg', 'audio/wav'];
  
  if (imageTypes.includes(this.type)) return 'image';
  if (videoTypes.includes(this.type)) return 'video';
  if (audioTypes.includes(this.type)) return 'audio';
  return 'file';
};

// Method to check if attachment is from server
AttachmentSchema.methods.isFromServer = function() {
  return this.conversationType === 'server';
};

// Method to check if attachment is from DM
AttachmentSchema.methods.isFromDM = function() {
  return this.conversationType === 'dm';
};

// Method to check if attachment is from group
AttachmentSchema.methods.isFromGroup = function() {
  return this.conversationType === 'group';
};

// Static method to get attachments by server
AttachmentSchema.statics.getServerAttachments = async function(serverId, options = {}) {
  const query = { server: serverId, conversationType: 'server' };
  if (options.type) query.type = options.type;
  
  return this.find(query)
    .populate('channel', 'name')
    .sort({ createdAt: -1 })
    .limit(options.limit || 50)
    .skip(options.skip || 0);
};

// Static method to get attachments by channel
AttachmentSchema.statics.getChannelAttachments = async function(channelId, options = {}) {
  const query = { channel: channelId };
  if (options.type) query.type = options.type;
  
  return this.find(query)
    .populate('server', 'name')
    .sort({ createdAt: -1 })
    .limit(options.limit || 50)
    .skip(options.skip || 0);
};

// Static method to get attachments by type
AttachmentSchema.statics.getAttachmentsByType = async function(type, options = {}) {
  const query = { type };
  if (options.server) query.server = options.server;
  if (options.channel) query.channel = options.channel;
  
  return this.find(query)
    .populate('server', 'name')
    .populate('channel', 'name')
    .sort({ createdAt: -1 })
    .limit(options.limit || 50)
    .skip(options.skip || 0);
};

// Static method to get recent attachments
AttachmentSchema.statics.getRecentAttachments = async function(options = {}) {
  const query = {};
  if (options.type) query.type = options.type;
  if (options.server) query.server = options.server;
  if (options.channel) query.channel = options.channel;
  
  return this.find(query)
    .populate('server', 'name')
    .populate('channel', 'name')
    .sort({ createdAt: -1 })
    .limit(options.limit || 50)
    .skip(options.skip || 0);
};

// Static method to create attachment
AttachmentSchema.statics.createAttachment = async function(data) {
  return this.create({
    url: data.url,
    type: data.type,
    size: data.size,
    conversationType: data.conversationType,
    channel: data.channel,
    server: data.server
  });
};

export default mongoose.model('Attachment', AttachmentSchema);