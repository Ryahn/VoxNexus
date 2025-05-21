import mongoose from 'mongoose';
import utility from '../../utils';

/**
 * Schema for server bans
 * Tracks user bans within servers
 */
const BanSchema = new mongoose.Schema({
  _id: {
    type: String,
    default: () => utility.generateSnowflake().toString()
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
  bannedBy: {
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
  reason: {
    type: String,
    maxlength: 1000,
    trim: true
  },
  expiresAt: {
    type: Date,
    default: null
  },
  isActive: {
    type: Boolean,
    default: true
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

// Create indexes
BanSchema.index({ server: 1, user: 1 }, { unique: true });
BanSchema.index({ server: 1, isActive: 1 });
BanSchema.index({ user: 1, isActive: 1 });
BanSchema.index({ bannedBy: 1 });
BanSchema.index({ expiresAt: 1 });
BanSchema.index({ createdAt: 1 });

// Update timestamp before saving
BanSchema.pre('save', function(next) {
  this.updatedAt = new Date();
  next();
});

// Method to check if ban is still active
BanSchema.methods.isActive = function() {
  if (!this.isActive) return false;
  if (this.expiresAt && new Date() > this.expiresAt) {
    this.isActive = false;
    return false;
  }
  return true;
};

// Method to update ban reason
BanSchema.methods.updateReason = async function(reason) {
  this.reason = reason;
  await this.save();
};

// Method to extend ban duration
BanSchema.methods.extendDuration = async function(days) {
  if (this.expiresAt) {
    this.expiresAt = new Date(this.expiresAt.getTime() + (days * 24 * 60 * 60 * 1000));
  } else {
    this.expiresAt = new Date(Date.now() + (days * 24 * 60 * 60 * 1000));
  }
  await this.save();
};

// Static method to get all active bans for a server
BanSchema.statics.getActiveBans = async function(serverId) {
  return this.find({
    server: serverId,
    isActive: true,
    $or: [
      { expiresAt: null },
      { expiresAt: { $gt: new Date() } }
    ]
  }).populate('user', 'username avatar')
    .populate('bannedBy', 'username');
};

// Static method to get all bans for a user
BanSchema.statics.getUserBans = async function(userId) {
  return this.find({ user: userId })
    .populate('server', 'name icon')
    .populate('bannedBy', 'username')
    .sort({ createdAt: -1 });
};

// Static method to get all bans issued by a user
BanSchema.statics.getBansByUser = async function(userId) {
  return this.find({ bannedBy: userId })
    .populate('user', 'username avatar')
    .populate('server', 'name icon')
    .sort({ createdAt: -1 });
};

// Static method to delete expired bans
BanSchema.statics.cleanupExpiredBans = async function() {
  return this.deleteMany({
    isActive: true,
    expiresAt: { $lt: new Date() }
  });
};

// Static method to create a ban
BanSchema.statics.createBan = async function(data) {
  return this.create({
    server: data.server,
    user: data.user,
    bannedBy: data.bannedBy,
    reason: data.reason,
    expiresAt: data.expiresAt
  });
};

export default mongoose.model('Ban', BanSchema); 