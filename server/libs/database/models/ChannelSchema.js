import mongoose from 'mongoose';
import utility from '../../utils';

const ChannelSchema = new mongoose.Schema({
  _id: {
    type: String,
    default: () => utility.generateSnowflake().toString()
  },
  name: {
    type: String,
    required: true,
    trim: true,
    minlength: 3,
    maxlength: 100,
    validate: {
      validator: function(v) {
        return /^[a-z0-9-]+$/.test(v);
      },
      message: 'Channel name can only contain lowercase letters, numbers, and hyphens'
    }
  },
  description: {
    type: String,
    maxlength: 1024
  },
  icon: {
    type: String,
    ref: 'Attachment',
    validate: {
      validator: function(v) {
        return !v || mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid attachment reference'
    }
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
  isNSFW: {
    type: Boolean,
    default: false
  },
  category: {
    type: String,
    default: 'general'
  },
  members: [{
    type: String,
    ref: 'User'
  }],
  rolePermissions: [{
    role: {
      type: String,
      ref: 'Role',
      required: true
    },
    permissions: {
      type: [String],
      default: []
    }
  }],
  memberPermissions: [{
    member: {
      type: String,
      ref: 'User',
      required: true
    },
    permissions: {
      type: [String],
      default: []
    }
  }],
  server: {
    type: String,
    ref: 'Server',
    required: true
  },
  type: {
    type: String,
    enum: ['text', 'forum', 'announcement', 'media', 'stage'],
    default: 'text'
  },
  position: {
    type: Number,
    default: 0
  },
  slowmode: {
    enabled: {
      type: Boolean,
      default: false
    },
    interval: {
      type: Number,
      default: 0,
      min: 0,
      max: 21600 // 6 hours in seconds
    }
  },
  rateLimit: {
    enabled: {
      type: Boolean,
      default: false
    },
    maxMessages: {
      type: Number,
      default: 0
    },
    timeWindow: {
      type: Number,
      default: 0
    }
  },
  isLocked: {
    type: Boolean,
    default: false
  },
  isArchived: {
    type: Boolean,
    default: false
  },
  messageRetention: {
    enabled: {
      type: Boolean,
      default: false
    },
    days: {
      type: Number,
      default: 0,
      min: 0,
      max: 365
    }
  },
  threadSettings: {
    enabled: {
      type: Boolean,
      default: true
    },
    autoArchiveDuration: {
      type: Number,
      enum: [60, 1440, 4320, 10080], // 1 hour, 1 day, 3 days, 1 week in minutes
      default: 1440
    },
    requireApproval: {
      type: Boolean,
      default: false
    }
  },
  lastMessageAt: {
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

// Compound indexes for common queries
ChannelSchema.index({ server: 1, category: 1, position: 1 });
ChannelSchema.index({ server: 1, type: 1, position: 1 });
ChannelSchema.index({ server: 1, isPrivate: 1 });
ChannelSchema.index({ server: 1, isNSFW: 1 });
ChannelSchema.index({ lastMessageAt: -1 });
ChannelSchema.index({ 'members': 1 });
ChannelSchema.index({ 'rolePermissions.role': 1 });
ChannelSchema.index({ 'memberPermissions.member': 1 });

// Virtual for message count
ChannelSchema.virtual('messageCount', {
  ref: 'Message',
  localField: '_id',
  foreignField: 'channel',
  count: true
});

// Virtual for thread count
ChannelSchema.virtual('threadCount', {
  ref: 'Thread',
  localField: '_id',
  foreignField: 'channel',
  count: true
});

// Pre-save middleware to ensure position is unique within server and category
ChannelSchema.pre('save', async function(next) {
  if (this.isModified('position')) {
    const Channel = mongoose.model('Channel');
    const existingChannel = await Channel.findOne({
      server: this.server,
      category: this.category,
      position: this.position,
      _id: { $ne: this._id }
    });
    
    if (existingChannel) {
      await Channel.updateMany(
        {
          server: this.server,
          category: this.category,
          position: { $gte: this.position },
          _id: { $ne: this._id }
        },
        { $inc: { position: 1 } }
      );
    }
  }
  next();
});

// Method to lock channel
ChannelSchema.methods.lock = async function() {
  this.isLocked = true;
  await this.save();
};

// Method to unlock channel
ChannelSchema.methods.unlock = async function() {
  this.isLocked = false;
  await this.save();
};

// Method to archive channel
ChannelSchema.methods.archive = async function() {
  this.isArchived = true;
  await this.save();
};

// Method to unarchive channel
ChannelSchema.methods.unarchive = async function() {
  this.isArchived = false;
  await this.save();
};

export default mongoose.model('Channel', ChannelSchema);