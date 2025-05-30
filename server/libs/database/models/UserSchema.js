import mongoose from 'mongoose';
import bcrypt from 'bcrypt';
import { utility } from '../../utils.js';

const RefreshTokenSchema = new mongoose.Schema({
    token: {
        type: String,
        required: true
    },
    expiresAt: {
        type: Date,
        required: true
    },
    createdAt: {
        type: Date,
        default: Date.now
    },
    deviceInfo: {
        type: String,
        default: 'Unknown'
    },
    isValid: {
        type: Boolean,
        default: true
    }
});

const UserSchema = new mongoose.Schema({
  _id: {
    type: String,
    default: () => utility.generateSnowflake().toString()
  },
  username: {
    type: String,
    required: true,
    maxlength: 32,
    trim: true,
    validate: {
      validator: function(v) {
        return /^[a-zA-Z0-9\s\-_]+$/.test(v);
      },
      message: 'Username can only contain letters, numbers, spaces, hyphens, and underscores'
    }
  },
  email: {
    type: String,
    required: true,
    maxlength: 255,
    trim: true,
    validate: {
      validator: function(v) {
        return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(v);
      },
      message: 'Invalid email address'
    }
  },
  password: {
    type: String,
    required: true
  },
  avatar: {
    type: String,
    ref: 'Attachment',
    default: null,
    validate: {
      validator: function(v) {
        return !v || mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid attachment reference'
    }
  },
  profile_banner: {
    type: String,
    ref: 'Attachment',
    default: null,
    validate: {
      validator: function(v) {
        return !v || mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid attachment reference'
    }
  },
  accent_color: {
    type: String,
    default: null,
    validate: {
      validator: function(v) {
        return !v || /^#([0-9a-fA-F]{6})$/.test(v);
      },
      message: 'Invalid accent color format'
    }
  },
  bio: {
    type: String,
    default: null,
    maxlength: 1000,
    trim: true
  },
  birthday: {
    type: Date,
    default: null,
    validate: {
      validator: function(v) {
        return !v || v instanceof Date;
      },
      message: 'Invalid birthday date'
    }
  },
  last_online: {
    type: Date,
    default: null,
    validate: {
      validator: function(v) {
        return !v || v instanceof Date;
      },
      message: 'Invalid last online date'
    }
  },
  is_verified: {
    type: Boolean,
    default: false
  },
  is_banned: {
    type: Boolean,
    default: false
  },
  ban_reason: {
    type: String,
    default: null
  },
  banned_at: {
    type: Date,
    default: null
  },
  is_deleted: {
    type: Boolean,
    default: false
  },
  deleted_at: {
    type: Date,
    default: null
  },
  serverRoles: [{
    server: {
      type: String,
      ref: 'Server',
      required: true
    },
    roles: [{
      type: String,
      ref: 'Role',
      required: true
    }]
  }],
  refreshTokens: [RefreshTokenSchema],
  verificationToken: {
    type: String,
    default: null
  },
  verificationTokenExpiry: {
    type: Date,
    default: null
  },
  passwordResetToken: {
    type: String,
    default: null
  },
  passwordResetTokenExpiry: {
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
  timestamps: true
});

// Create indexes
UserSchema.index({ username: 1 }, { unique: true });
UserSchema.index({ email: 1 }, { unique: true });
UserSchema.index({ is_verified: 1, is_banned: 1 });
UserSchema.index({ last_online: 1, birthday: 1 });
UserSchema.index({ accent_color: 1, bio: 1 });
UserSchema.index({ profile_banner: 1, avatar: 1 });

// Method to ban a user
UserSchema.methods.ban = async function(reason = '') {
  this.is_banned = true;
  this.ban_reason = reason;
  this.banned_at = new Date();
  await this.save();
};

// Method to unban a user
UserSchema.methods.unban = async function() {
  this.is_banned = false;
  this.ban_reason = null;
  this.banned_at = null;
  await this.save();
};

// Method to soft delete a user
UserSchema.methods.softDelete = async function() {
  this.is_deleted = true;
  this.deleted_at = new Date();
  // Clear sensitive data
  this.email = `deleted_${this._id}@deleted.user`;
  this.password = 'deleted';
  this.avatar = null;
  this.profile_banner = null;
  this.bio = null;
  this.accent_color = null;
  this.birthday = null;
  await this.save();
};

// Method to restore a soft-deleted user
UserSchema.methods.restore = async function(email, password) {
  this.is_deleted = false;
  this.deleted_at = null;
  this.email = email;
  this.password = password;
  await this.save();
};

// Method to hard delete a user and all their related data
UserSchema.methods.hardDelete = async function() {
  const Message = mongoose.model('Message');
  const DirectMessage = mongoose.model('DirectMessage');
  const Server = mongoose.model('Server');
  const Channel = mongoose.model('Channel');
  const Role = mongoose.model('Role');
  const Ban = mongoose.model('Ban');
  const AuditLog = mongoose.model('AuditLog');
  const Webhook = mongoose.model('Webhook');
  const Invite = mongoose.model('Invite');
  
  // Delete all related data
  await Promise.all([
    // Delete user's messages
    Message.updateMany(
      { sender: this._id },
      { $set: { deleted: true, content: '[deleted]' } }
    ),
    // Remove user from direct messages
    DirectMessage.updateMany(
      { participants: this._id },
      { $pull: { participants: this._id } }
    ),
    // Remove user from servers
    Server.updateMany(
      { members: this._id },
      { $pull: { members: this._id } }
    ),
    // Remove user from channels
    Channel.updateMany(
      { members: this._id },
      { $pull: { members: this._id } }
    ),
    // Remove user's roles
    Role.updateMany(
      { 'members.user': this._id },
      { $pull: { 'members': { user: this._id } } }
    ),
    // Delete user's bans
    Ban.deleteMany({ user: this._id }),
    // Delete user's audit logs
    AuditLog.deleteMany({ user: this._id }),
    // Delete user's webhooks
    Webhook.deleteMany({ createdBy: this._id }),
    // Delete user's invites
    Invite.deleteMany({ createdBy: this._id })
  ]);
  
  // Finally delete the user
  await this.deleteOne();
};

export default mongoose.model('User', UserSchema);