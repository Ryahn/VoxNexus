import mongoose from 'mongoose';
import { utility } from '../../utils.js';

const ThreadSchema = new mongoose.Schema({
  _id: {
    type: String,
    default: () => utility.generateSnowflake().toString()
  },
  name: {
    type: String,
    required: true,
    trim: true,
    minlength: 1,
    maxlength: 100
  },
  // Thread can be in a channel, forum post, or direct message
  channel: {
    type: String,
    ref: 'Channel'
  },
  forumPost: {
    type: String,
    ref: 'ForumPost'
  },
  directMessage: {
    type: String,
    ref: 'DirectMessage'
  },
  // Original message that started the thread
  parentMessage: {
    type: String,
    ref: 'Message',
    required: true
  },
  createdBy: {
    type: String,
    ref: 'User',
    required: true
  },
  isPrivate: {
    type: Boolean,
    default: false
  },
  isArchived: {
    type: Boolean,
    default: false
  },
  isLocked: {
    type: Boolean,
    default: false
  },
  autoArchiveDuration: {
    type: Number,
    enum: [60, 1440, 4320, 10080, 20160, 43200], // 1 hour, 1 day, 3 days, 1 week, 2 weeks, 30 days in minutes
    default: 1440
  },
  memberCount: {
    type: Number,
    default: 1
  },
  messageCount: {
    type: Number,
    default: 0
  },
  lastMessageAt: {
    type: Date,
    default: Date.now
  },
  lastMessageId: {
    type: String,
    ref: 'Message'
  },
  members: [{
    user: {
      type: String,
      ref: 'User',
      required: true
    },
    joinedAt: {
      type: Date,
      default: Date.now
    },
    notifications: {
      type: Boolean,
      default: true
    }
  }],
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

// Indexes for common queries
ThreadSchema.index({ channel: 1, lastMessageAt: -1 });
ThreadSchema.index({ forumPost: 1, parentMessage: 1 });
ThreadSchema.index({ directMessage: 1, createdBy: 1 });
ThreadSchema.index({ 'members.user': 1 });
ThreadSchema.index({ isArchived: 1 });
ThreadSchema.index({ isLocked: 1 });

// Virtual for messages
ThreadSchema.virtual('messages', {
  ref: 'Message',
  localField: '_id',
  foreignField: 'thread'
});

// Validation to ensure thread is associated with exactly one parent
ThreadSchema.pre('save', function(next) {
  const parentCount = [this.channel, this.forumPost, this.directMessage].filter(Boolean).length;
  if (parentCount !== 1) {
    next(new Error('Thread must be associated with exactly one parent (channel, forum post, or direct message)'));
  }
  next();
});

// Pre-save middleware to update lastMessageAt
ThreadSchema.pre('save', async function(next) {
  if (this.isModified('lastMessageId')) {
    this.lastMessageAt = new Date();
  }
  next();
});

// Method to add a member to the thread
ThreadSchema.methods.addMember = async function(userId) {
  if (!this.members.some(m => m.user.toString() === userId.toString())) {
    this.members.push({
      user: userId,
      joinedAt: new Date(),
      notifications: true
    });
    this.memberCount = this.members.length;
    await this.save();
  }
};

// Method to remove a member from the thread
ThreadSchema.methods.removeMember = async function(userId) {
  this.members = this.members.filter(m => m.user.toString() !== userId.toString());
  this.memberCount = this.members.length;
  await this.save();
};

// Method to toggle member notifications
ThreadSchema.methods.toggleNotifications = async function(userId) {
  const member = this.members.find(m => m.user.toString() === userId.toString());
  if (member) {
    member.notifications = !member.notifications;
    await this.save();
  }
};

// Method to archive thread
ThreadSchema.methods.archive = async function() {
  this.isArchived = true;
  await this.save();
};

// Method to unarchive thread
ThreadSchema.methods.unarchive = async function() {
  this.isArchived = false;
  await this.save();
};

// Method to lock thread
ThreadSchema.methods.lock = async function() {
  this.isLocked = true;
  await this.save();
};

// Method to unlock thread
ThreadSchema.methods.unlock = async function() {
  this.isLocked = false;
  await this.save();
};

export default mongoose.model('Thread', ThreadSchema); 