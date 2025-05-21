import mongoose from 'mongoose';
import utility from '../../utils';

/**
 * Schema for server invites
 * Manages invite codes and their usage
 */
const InviteSchema = new mongoose.Schema({
  _id: {
    type: String,
    default: () => utility.generateSnowflake().toString()
  },
  code: {
    type: String,
    required: true,
    unique: true,
    trim: true,
    validate: {
      validator: function(v) {
        return /^[a-zA-Z0-9]{8,}$/.test(v);
      },
      message: 'Invalid invite code format'
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
  createdBy: {
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
  maxUses: {
    type: Number,
    default: 0, // 0 means unlimited
    min: 0
  },
  currentUses: {
    type: Number,
    default: 0,
    min: 0
  },
  expiresAt: {
    type: Date,
    default: null
  },
  isTemporary: {
    type: Boolean,
    default: false
  },
  temporaryExpiresAt: {
    type: Date,
    default: null,
    validate: {
      validator: function(v) {
        return !this.isTemporary || (this.isTemporary && v instanceof Date);
      },
      message: 'Temporary invites must have an expiration date'
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
  timestamps: true
});

// Create indexes
InviteSchema.index({ code: 1 }, { unique: true });
InviteSchema.index({ server: 1 });
InviteSchema.index({ createdBy: 1 });
InviteSchema.index({ expiresAt: 1 });
InviteSchema.index({ temporaryExpiresAt: 1 });
InviteSchema.index({ currentUses: 1 });

// Generate a random invite code before saving
InviteSchema.pre('save', function(next) {
  if (!this.code) {
    this.code = utility.generateInviteCode();
  }
  this.updatedAt = new Date();
  next();
});

// Method to check if invite is valid
InviteSchema.methods.isValid = function() {
  if (this.expiresAt && new Date() > this.expiresAt) return false;
  if (this.maxUses > 0 && this.currentUses >= this.maxUses) return false;
  if (this.isTemporary && this.temporaryExpiresAt && new Date() > this.temporaryExpiresAt) return false;
  return true;
};

// Method to increment use count
InviteSchema.methods.incrementUses = async function() {
  if (this.isValid()) {
    this.currentUses += 1;
    await this.save();
    return true;
  }
  return false;
};

// Method to revoke invite
InviteSchema.methods.revoke = async function() {
  this.expiresAt = new Date();
  await this.save();
};

// Static method to get all active invites for a server
InviteSchema.statics.getActiveInvites = async function(serverId) {
  return this.find({
    server: serverId,
    $or: [
      { expiresAt: null },
      { expiresAt: { $gt: new Date() } }
    ],
    $or: [
      { maxUses: 0 },
      { currentUses: { $lt: '$maxUses' } }
    ]
  }).populate('createdBy', 'username');
};

// Static method to get all invites created by a user
InviteSchema.statics.getInvitesByUser = async function(userId) {
  return this.find({ createdBy: userId })
    .populate('server', 'name icon')
    .sort({ createdAt: -1 });
};

// Static method to revoke all invites for a server
InviteSchema.statics.revokeAllForServer = async function(serverId) {
  return this.updateMany(
    { server: serverId, expiresAt: null },
    { expiresAt: new Date() }
  );
};

// Static method to revoke all invites created by a user
InviteSchema.statics.revokeAllByUser = async function(userId) {
  return this.updateMany(
    { createdBy: userId, expiresAt: null },
    { expiresAt: new Date() }
  );
};

export default mongoose.model('Invite', InviteSchema); 