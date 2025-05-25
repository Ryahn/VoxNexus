import mongoose from 'mongoose';
import { utility } from '../../utils.js';

/**
 * Schema for direct messages between users
 * Handles both 1-on-1 and group direct messages
 */
const DirectMessageSchema = new mongoose.Schema({
  _id: {
    type: String,
    default: () => utility.generateSnowflake().toString()
  },
  participants: [{
    type: String,
    ref: 'User',
    required: true,
    validate: {
      validator: function(v) {
        return mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid user reference'
    }
  }],
  isGroup: {
    type: Boolean,
    default: false
  },
  name: {
    type: String,
    maxlength: 100,
    trim: true,
    validate: {
      validator: function(v) {
        return !this.isGroup || (this.isGroup && v && v.length > 0);
      },
      message: 'Group DMs must have a name'
    }
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
  owner: {
    type: String,
    ref: 'User',
    validate: {
      validator: function(v) {
        return !this.isGroup || (this.isGroup && mongoose.Types.ObjectId.isValid(v));
      },
      message: 'Invalid owner reference'
    }
  },
  messages: [{
    type: String,
    ref: 'Message',
    validate: {
      validator: function(v) {
        return mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid message reference'
    }
  }],
  lastMessage: {
    type: String,
    ref: 'Message',
    validate: {
      validator: function(v) {
        return !v || mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid message reference'
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
    }
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

// Create indexes
DirectMessageSchema.index({ participants: 1 });
DirectMessageSchema.index({ 'lastMessage': 1 });
DirectMessageSchema.index({ isGroup: 1 });

// Virtual for message count
DirectMessageSchema.virtual('messageCount', {
  ref: 'Message',
  localField: '_id',
  foreignField: 'directMessage',
  count: true
});

// Virtual for thread count
DirectMessageSchema.virtual('threadCount', {
  ref: 'Thread',
  localField: '_id',
  foreignField: 'directMessage',
  count: true
});

// Update lastMessage before saving
DirectMessageSchema.pre('save', function(next) {
  if (this.messages.length > 0) {
    this.lastMessage = this.messages[this.messages.length - 1];
  }
  this.updatedAt = new Date();
  next();
});

// Method to add participant
DirectMessageSchema.methods.addParticipant = async function(userId) {
  if (!this.participants.includes(userId)) {
    this.participants.push(userId);
    await this.save();
  }
};

// Method to remove participant
DirectMessageSchema.methods.removeParticipant = async function(userId) {
  this.participants = this.participants.filter(p => p.toString() !== userId.toString());
  if (this.participants.length === 0) {
    await this.remove();
  } else {
    await this.save();
  }
};

// Method to update group name
DirectMessageSchema.methods.updateName = async function(name) {
  if (this.isGroup) {
    this.name = name;
    await this.save();
  }
};

// Method to update group icon
DirectMessageSchema.methods.updateIcon = async function(iconId) {
  if (this.isGroup) {
    this.icon = iconId;
    await this.save();
  }
};

// Method to transfer ownership
DirectMessageSchema.methods.transferOwnership = async function(newOwnerId) {
  if (this.isGroup && this.participants.includes(newOwnerId)) {
    this.owner = newOwnerId;
    await this.save();
  }
};

// Method to update thread settings
DirectMessageSchema.methods.updateThreadSettings = async function(settings) {
  Object.assign(this.threadSettings, settings);
  await this.save();
};

// Static method to get user's DMs
DirectMessageSchema.statics.getUserDMs = async function(userId) {
  return this.find({ participants: userId })
    .populate('participants', 'username avatar')
    .populate('owner', 'username avatar')
    .populate('lastMessage')
    .sort({ updatedAt: -1 });
};

// Static method to get group DMs
DirectMessageSchema.statics.getGroupDMs = async function(userId) {
  return this.find({
    participants: userId,
    isGroup: true
  })
    .populate('participants', 'username avatar')
    .populate('owner', 'username avatar')
    .populate('lastMessage')
    .sort({ updatedAt: -1 });
};

// Static method to find DM between users
DirectMessageSchema.statics.findDM = async function(userId1, userId2) {
  return this.findOne({
    participants: { $all: [userId1, userId2] },
    isGroup: false
  })
    .populate('participants', 'username avatar')
    .populate('lastMessage');
};

// Static method to create group DM
DirectMessageSchema.statics.createGroupDM = async function(data) {
  return this.create({
    participants: data.participants,
    isGroup: true,
    name: data.name,
    icon: data.icon,
    owner: data.owner,
    threadSettings: data.threadSettings
  });
};

export default mongoose.model('DirectMessage', DirectMessageSchema); 