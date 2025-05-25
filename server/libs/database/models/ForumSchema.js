import mongoose from 'mongoose';
import { utility } from '../../utils.js';

const ForumSchema = new mongoose.Schema({
  _id: {
    type: String,
    default: () => utility.generateSnowflake().toString()
  },
  name: {
    type: String,
    required: true,
    trim: true,
    minlength: 3,
    maxlength: 100
  },
  description: {
    type: String,
    maxlength: 1024
  },
  icon: {
    type: String,
    ref: 'Attachment',
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
  // Forum-specific fields
  tags: [{
    name: {
      type: String,
      required: true,
      trim: true
    },
    color: {
      type: String,
      default: '#5865F2' // Discord's default blue color
    },
    emoji: {
      type: String
    }
  }],
  defaultTags: [{
    type: String,
    ref: 'Tag'
  }],
  postCount: {
    type: Number,
    default: 0
  },
  lastPostAt: {
    type: Date,
    default: null
  },
  isLocked: {
    type: Boolean,
    default: false
  },
  isArchived: {
    type: Boolean,
    default: false
  },
  sortOrder: {
    type: String,
    enum: ['latest', 'most_relevant', 'most_reactions'],
    default: 'latest'
  },
  channel: {
    type: String,
    ref: 'Channel',
    required: true
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

// Indexes for common queries
ForumSchema.index({ channel: 1 });
ForumSchema.index({ lastPostAt: -1 });
ForumSchema.index({ postCount: -1 });
ForumSchema.index({ isArchived: 1 });
ForumSchema.index({ isLocked: 1 });

// Virtual for posts
ForumSchema.virtual('posts', {
  ref: 'ForumPost',
  localField: '_id',
  foreignField: 'forum'
});

// Method to add a tag
ForumSchema.methods.addTag = async function(tagData) {
  if (!this.tags.some(t => t.name === tagData.name)) {
    this.tags.push(tagData);
    await this.save();
  }
};

// Method to remove a tag
ForumSchema.methods.removeTag = async function(tagName) {
  this.tags = this.tags.filter(t => t.name !== tagName);
  await this.save();
};

// Method to update tag
ForumSchema.methods.updateTag = async function(tagName, updates) {
  const tag = this.tags.find(t => t.name === tagName);
  if (tag) {
    Object.assign(tag, updates);
    await this.save();
  }
};

// Method to lock forum
ForumSchema.methods.lock = async function() {
  this.isLocked = true;
  await this.save();
};

// Method to unlock forum
ForumSchema.methods.unlock = async function() {
  this.isLocked = false;
  await this.save();
};

// Method to archive forum
ForumSchema.methods.archive = async function() {
  this.isArchived = true;
  await this.save();
};

// Method to unarchive forum
ForumSchema.methods.unarchive = async function() {
  this.isArchived = false;
  await this.save();
};

// Static method to get forums by channel
ForumSchema.statics.getForumsByChannel = async function(channelId) {
  return this.find({ channel: channelId })
    .populate('createdBy', 'username avatar')
    .sort({ position: 1 });
};

// Static method to get active forums
ForumSchema.statics.getActiveForums = async function() {
  return this.find({ isArchived: false, isLocked: false })
    .populate('createdBy', 'username avatar')
    .sort({ lastPostAt: -1 });
};

// Static method to get forums by tag
ForumSchema.statics.getForumsByTag = async function(tagName) {
  return this.find({ 'tags.name': tagName })
    .populate('createdBy', 'username avatar')
    .sort({ lastPostAt: -1 });
};

export default mongoose.model('Forum', ForumSchema);