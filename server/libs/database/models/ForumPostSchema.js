import mongoose from 'mongoose';
import { utility } from '../../utils.js';

const ForumPostSchema = new mongoose.Schema({
  _id: {
    type: String,
    default: () => utility.generateSnowflake().toString()
  },
  title: {
    type: String,
    required: true,
    trim: true,
    minlength: 1,
    maxlength: 100
  },
  content: {
    type: String,
    required: true,
    trim: true,
    maxlength: 4000
  },
  author: {
    type: String,
    ref: 'User',
    required: true
  },
  forum: {
    type: String,
    ref: 'Forum',
    required: true
  },
  tags: [{
    type: String,
    ref: 'Tag'
  }],
  isPinned: {
    type: Boolean,
    default: false
  },
  isLocked: {
    type: Boolean,
    default: false
  },
  isSolved: {
    type: Boolean,
    default: false
  },
  solution: {
    type: String,
    ref: 'ForumPostReply'
  },
  viewCount: {
    type: Number,
    default: 0
  },
  reactionCount: {
    type: Number,
    default: 0
  },
  replyCount: {
    type: Number,
    default: 0
  },
  lastReplyAt: {
    type: Date,
    default: null
  },
  lastReplyBy: {
    type: String,
    ref: 'User'
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
ForumPostSchema.index({ forum: 1, createdAt: -1 });
ForumPostSchema.index({ isSolved: 1, isPinned: -1});
ForumPostSchema.index({ author: 1 });
ForumPostSchema.index({ lastReplyAt: -1 });
ForumPostSchema.index({ viewCount: -1 });
ForumPostSchema.index({ reactionCount: -1 });

// Virtual for replies
ForumPostSchema.virtual('replies', {
  ref: 'ForumPostReply',
  localField: '_id',
  foreignField: 'post'
});

// Virtual for reactions
ForumPostSchema.virtual('reactions', {
  ref: 'Reaction',
  localField: '_id',
  foreignField: 'post'
});

// Method to increment view count
ForumPostSchema.methods.incrementViews = async function() {
  this.viewCount += 1;
  await this.save();
};

// Method to toggle pin status
ForumPostSchema.methods.togglePin = async function() {
  this.isPinned = !this.isPinned;
  await this.save();
};

// Method to toggle lock status
ForumPostSchema.methods.toggleLock = async function() {
  this.isLocked = !this.isLocked;
  await this.save();
};

// Method to mark as solved
ForumPostSchema.methods.markAsSolved = async function(solutionId) {
  this.isSolved = true;
  this.solution = solutionId;
  await this.save();
};

// Method to mark as unsolved
ForumPostSchema.methods.markAsUnsolved = async function() {
  this.isSolved = false;
  this.solution = null;
  await this.save();
};

// Method to update tags
ForumPostSchema.methods.updateTags = async function(tagIds) {
  this.tags = tagIds;
  await this.save();
};

// Static method to get posts by forum
ForumPostSchema.statics.getPostsByForum = async function(forumId, options = {}) {
  const query = { forum: forumId };
  if (options.isPinned) query.isPinned = true;
  if (options.isSolved) query.isSolved = true;
  
  return this.find(query)
    .populate('author', 'username avatar')
    .populate('lastReplyBy', 'username avatar')
    .sort({ isPinned: -1, createdAt: -1 })
    .limit(options.limit || 20)
    .skip(options.skip || 0);
};

// Static method to get posts by author
ForumPostSchema.statics.getPostsByAuthor = async function(authorId) {
  return this.find({ author: authorId })
    .populate('forum', 'name')
    .sort({ createdAt: -1 });
};

// Static method to get solved posts
ForumPostSchema.statics.getSolvedPosts = async function(forumId) {
  return this.find({ forum: forumId, isSolved: true })
    .populate('author', 'username avatar')
    .populate('solution')
    .sort({ updatedAt: -1 });
};

// Static method to get pinned posts
ForumPostSchema.statics.getPinnedPosts = async function(forumId) {
  return this.find({ forum: forumId, isPinned: true })
    .populate('author', 'username avatar')
    .sort({ updatedAt: -1 });
};

// Pre-save middleware to update forum's lastPostAt
ForumPostSchema.pre('save', async function(next) {
  if (this.isNew) {
    const Forum = mongoose.model('Forum');
    await Forum.findByIdAndUpdate(this.forum, {
      $inc: { postCount: 1 },
      lastPostAt: new Date()
    });
  }
  next();
});

// Pre-remove middleware to update forum's postCount
ForumPostSchema.pre('remove', async function(next) {
  const Forum = mongoose.model('Forum');
  await Forum.findByIdAndUpdate(this.forum, {
    $inc: { postCount: -1 }
  });
  next();
});

export default mongoose.model('ForumPost', ForumPostSchema); 