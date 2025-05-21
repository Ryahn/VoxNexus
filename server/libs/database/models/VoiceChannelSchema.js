import mongoose from 'mongoose';
import utility from '../../utils';

/**
 * Schema for voice channels
 * Manages voice channel settings and participants
 */
const VoiceChannelSchema = new mongoose.Schema({
  _id: {
    type: String,
    default: () => utility.generateSnowflake().toString()
  },
  name: {
    type: String,
    required: true,
    maxlength: 100,
    trim: true,
    validate: {
      validator: function(v) {
        return /^[a-zA-Z0-9\s\-_]+$/.test(v);
      },
      message: 'Channel name can only contain letters, numbers, spaces, hyphens, and underscores'
    }
  },
  server: {
    type: String,
    ref: 'Server',
    required: true,
    validate: {
      validator: function(v) {
        return mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid server reference'
    }
  },
  category: {
    type: String,
    ref: 'Category',
    validate: {
      validator: function(v) {
        return !v || mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid category reference'
    }
  },
  position: {
    type: Number,
    required: true,
    min: 0
  },
  userLimit: {
    type: Number,
    default: 0, // 0 means no limit
    min: 0,
    max: 99
  },
  bitrate: {
    type: Number,
    default: 64000, // 64 kbps
    min: 8000,
    max: 384000
  },
  region: {
    type: String,
    default: 'auto',
    enum: ['auto', 'us-west', 'us-east', 'us-central', 'us-south', 'singapore', 'southafrica', 'sydney', 'rotterdam', 'russia', 'japan', 'india', 'hongkong', 'brazil']
  },
  participants: [{
    user: {
      type: String,
      ref: 'User',
      required: true,
      validate: {
        validator: function(v) {
          return mongoose.Types.ObjectId.isValid(v);
        },
        message: 'Invalid user reference'
      }
    },
    joinedAt: {
      type: Date,
      default: Date.now
    },
    isMuted: {
      type: Boolean,
      default: false
    },
    isDeafened: {
      type: Boolean,
      default: false
    }
  }],
  isPrivate: {
    type: Boolean,
    default: false
  },
  allowedRoles: [{
    type: String,
    ref: 'Role',
    validate: {
      validator: function(v) {
        return mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid role reference'
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
  timestamps: true
});

// Create indexes
VoiceChannelSchema.index({ server: 1, position: 1 });
VoiceChannelSchema.index({ server: 1, name: 1 });
VoiceChannelSchema.index({ category: 1 });
VoiceChannelSchema.index({ 'participants.user': 1 });
VoiceChannelSchema.index({ isPrivate: 1 });

// Update timestamp before saving
VoiceChannelSchema.pre('save', function(next) {
  this.updatedAt = new Date();
  next();
});

// Method to check if user can join the channel
VoiceChannelSchema.methods.canUserJoin = function(userId, userRoles) {
  if (this.isPrivate) {
    if (!this.allowedRoles.length) return false;
    return userRoles.some(role => this.allowedRoles.includes(role));
  }
  return true;
};

// Method to add participant to channel
VoiceChannelSchema.methods.addParticipant = async function(userId) {
  if (!this.participants.some(p => p.user.toString() === userId.toString())) {
    this.participants.push({
      user: userId,
      joinedAt: new Date(),
      isMuted: false,
      isDeafened: false
    });
    await this.save();
  }
};

// Method to remove participant from channel
VoiceChannelSchema.methods.removeParticipant = async function(userId) {
  this.participants = this.participants.filter(p => p.user.toString() !== userId.toString());
  await this.save();
};

// Method to toggle mute for participant
VoiceChannelSchema.methods.toggleMute = async function(userId) {
  const participant = this.participants.find(p => p.user.toString() === userId.toString());
  if (participant) {
    participant.isMuted = !participant.isMuted;
    await this.save();
  }
};

// Method to toggle deafen for participant
VoiceChannelSchema.methods.toggleDeafen = async function(userId) {
  const participant = this.participants.find(p => p.user.toString() === userId.toString());
  if (participant) {
    participant.isDeafened = !participant.isDeafened;
    await this.save();
  }
};

// Method to update channel settings
VoiceChannelSchema.methods.updateSettings = async function(settings) {
  const allowedUpdates = ['name', 'userLimit', 'bitrate', 'region'];
  Object.keys(settings).forEach(key => {
    if (allowedUpdates.includes(key)) {
      this[key] = settings[key];
    }
  });
  await this.save();
};

// Static method to get all voice channels for a server
VoiceChannelSchema.statics.getServerChannels = async function(serverId) {
  return this.find({ server: serverId })
    .populate('participants.user', 'username avatar')
    .sort({ position: 1 });
};

// Static method to get all voice channels a user is in
VoiceChannelSchema.statics.getUserChannels = async function(userId) {
  return this.find({ 'participants.user': userId })
    .populate('server', 'name icon')
    .sort({ position: 1 });
};

// Static method to delete all voice channels for a server
VoiceChannelSchema.statics.deleteAllForServer = async function(serverId) {
  return this.deleteMany({ server: serverId });
};

export default mongoose.model('VoiceChannel', VoiceChannelSchema); 