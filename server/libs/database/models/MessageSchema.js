import mongoose from 'mongoose';
import utility from '../../utils';

const MessageSchema = new mongoose.Schema({
  _id: {
    type: String,
    default: () => utility.generateSnowflake().toString()
  },
  content: {
    type: String,
    required: true,
    maxlength: 4000, // Discord-like message length limit
    trim: true
  },
  type: {
    type: String,
    enum: ['text', 'system', 'announcement', 'thread_created'],
    default: 'text'
  },
  channel: {
    type: String,
    ref: 'Channel',
    required: true
  },
  thread: {
    type: String,
    ref: 'Thread'
  },
  sender: {
    type: String,
    ref: 'User',
    required: true
  },
  attachments: [{
    type: String,
    ref: 'Attachment'
  }],
  reactions: [{
    type: String,
    ref: 'Reaction'
  }],
  mentions: [{
    type: String,
    ref: 'User'
  }],
  deleted: {
    type: Boolean,
    default: false
  },
  edited: {
    type: Boolean,
    default: false
  },
  editedAt: {
    type: Date,
    default: null
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
  timestamps: true,
  toJSON: { virtuals: true },
  toObject: { virtuals: true }
});

// Update the updatedAt timestamp before saving
MessageSchema.pre('save', function(next) {
  this.updatedAt = new Date();
  if (this.isModified('content')) {
    this.edited = true;
    this.editedAt = new Date();
  }
  next();
});

// Method to soft delete message
MessageSchema.methods.softDelete = async function() {
  this.deleted = true;
  this.content = '[Message deleted]';
  await this.save();
};

// Method to restore message
MessageSchema.methods.restore = async function() {
  this.deleted = false;
  await this.save();
};

// Method to add attachment
MessageSchema.methods.addAttachment = async function(attachmentId) {
  if (!this.attachments.includes(attachmentId)) {
    this.attachments.push(attachmentId);
    await this.save();
  }
};

// Method to remove attachment
MessageSchema.methods.removeAttachment = async function(attachmentId) {
  this.attachments = this.attachments.filter(a => a.toString() !== attachmentId.toString());
  await this.save();
};

// Method to add mention
MessageSchema.methods.addMention = async function(userId) {
  if (!this.mentions.includes(userId)) {
    this.mentions.push(userId);
    await this.save();
  }
};

// Method to remove mention
MessageSchema.methods.removeMention = async function(userId) {
  this.mentions = this.mentions.filter(m => m.toString() !== userId.toString());
  await this.save();
};

// Static method to get messages by channel
MessageSchema.statics.getMessagesByChannel = async function(channelId, options = {}) {
  const query = { channel: channelId, deleted: false };
  if (options.before) query.createdAt = { $lt: options.before };
  if (options.after) query.createdAt = { $gt: options.after };
  
  return this.find(query)
    .populate('sender', 'username avatar')
    .populate('attachments')
    .populate('mentions', 'username avatar')
    .sort({ createdAt: -1 })
    .limit(options.limit || 50)
    .skip(options.skip || 0);
};

// Static method to get messages by thread
MessageSchema.statics.getMessagesByThread = async function(threadId, options = {}) {
  const query = { thread: threadId, deleted: false };
  if (options.before) query.createdAt = { $lt: options.before };
  if (options.after) query.createdAt = { $gt: options.after };
  
  return this.find(query)
    .populate('sender', 'username avatar')
    .populate('attachments')
    .populate('mentions', 'username avatar')
    .sort({ createdAt: -1 })
    .limit(options.limit || 50)
    .skip(options.skip || 0);
};

// Static method to get messages by user
MessageSchema.statics.getMessagesByUser = async function(userId, options = {}) {
  const query = { sender: userId, deleted: false };
  if (options.channel) query.channel = options.channel;
  if (options.thread) query.thread = options.thread;
  
  return this.find(query)
    .populate('channel', 'name')
    .populate('thread', 'name')
    .populate('attachments')
    .sort({ createdAt: -1 })
    .limit(options.limit || 50)
    .skip(options.skip || 0);
};

// Static method to search messages
MessageSchema.statics.searchMessages = async function(query, options = {}) {
  const searchQuery = {
    content: { $regex: query, $options: 'i' },
    deleted: false
  };
  if (options.channel) searchQuery.channel = options.channel;
  if (options.thread) searchQuery.thread = options.thread;
  if (options.user) searchQuery.sender = options.user;
  
  return this.find(searchQuery)
    .populate('sender', 'username avatar')
    .populate('channel', 'name')
    .populate('thread', 'name')
    .sort({ createdAt: -1 })
    .limit(options.limit || 50)
    .skip(options.skip || 0);
};

// Indexes for common queries
MessageSchema.index({ channel: 1, createdAt: -1 });
MessageSchema.index({ thread: 1, createdAt: -1 });
MessageSchema.index({ type: 1, deleted: 1 });
MessageSchema.index({ sender: 1, mentions: 1 });

// Virtual for thread replies count
MessageSchema.virtual('replyCount', {
  ref: 'Message',
  localField: '_id',
  foreignField: 'thread',
  count: true
});

export default mongoose.model('Message', MessageSchema);