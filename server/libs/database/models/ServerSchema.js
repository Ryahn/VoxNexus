import mongoose from 'mongoose';
import { utility } from '../../utils.js';

const ServerSchema = new mongoose.Schema({
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
      message: 'Server name can only contain letters, numbers, spaces, hyphens, and underscores'
    }
  },
  description: {
    type: String,
    maxlength: 1000,
    trim: true
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
  banner: {
    type: String,
    ref: 'Attachment',
    validate: {
      validator: function(v) {
        return !v || mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid attachment reference'
    }
  },
  type: {
    type: String,
    enum: ['public', 'private', 'community'],
    default: 'public'
  },
  status: {
    type: String,
    enum: ['active', 'archived', 'deleted'],
    default: 'active'
  },
  owner: {
    type: String,
    ref: 'User',
    required: true
  },
  members: [{
    type: String,
    ref: 'User',
    validate: {
      validator: function(v) {
        return mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid member reference'
    }
  }],
  memberCount: {
    type: Number,
    default: 0
  },
  channels: [{
    type: String,
    ref: 'Channel',
    validate: {
      validator: function(v) {
        return mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid channel reference'
    }
  }],
  roles: [{
    type: String,
    ref: 'Role',
    validate: {
      validator: function(v) {
        return mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid role reference'
    }
  }],
  isDeleted: {
    type: Boolean,
    default: false
  },
  deletedAt: {
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
  timestamps: true // Automatically manage createdAt and updatedAt
});

// Update memberCount before saving
ServerSchema.pre('save', function(next) {
  this.memberCount = this.members.length;
  this.updatedAt = new Date();
  next();
});

// Create indexes
ServerSchema.index({ name: 1 }, { unique: true });
ServerSchema.index({ owner: 1 }, { unique: true });
ServerSchema.index({ type: 1, status: 1 });
ServerSchema.index({ icon: 1, banner: 1 });
ServerSchema.index({ description: 1 });

// Method to ban a user from the server
ServerSchema.methods.banUser = async function(userId, reason = '') {
  const Ban = mongoose.model('Ban');
  const existingBan = await Ban.findOne({ server: this._id, user: userId });
  
  if (!existingBan) {
    await Ban.create({
      server: this._id,
      user: userId,
      reason,
      bannedBy: this.owner // Assuming the server owner is doing the banning
    });
  }
  
  // Remove user from members array
  this.members = this.members.filter(member => member.toString() !== userId.toString());
  this.memberCount = this.members.length;
  await this.save();
};

// Method to unban a user from the server
ServerSchema.methods.unbanUser = async function(userId) {
  const Ban = mongoose.model('Ban');
  await Ban.findOneAndUpdate(
    { server: this._id, user: userId },
    { isActive: false }
  );
};

// Method to soft delete the server
ServerSchema.methods.softDelete = async function() {
  this.isDeleted = true;
  this.deletedAt = new Date();
  this.status = 'deleted';
  await this.save();
};

// Method to restore a soft-deleted server
ServerSchema.methods.restore = async function() {
  this.isDeleted = false;
  this.deletedAt = null;
  this.status = 'active';
  await this.save();
};

// Method to hard delete the server and all its related data
ServerSchema.methods.hardDelete = async function() {
  const Channel = mongoose.model('Channel');
  const Role = mongoose.model('Role');
  const Ban = mongoose.model('Ban');
  const AuditLog = mongoose.model('AuditLog');
  const Invite = mongoose.model('Invite');
  const Webhook = mongoose.model('Webhook');
  
  // Delete all related data
  await Promise.all([
    Channel.deleteMany({ server: this._id }),
    Role.deleteMany({ server: this._id }),
    Ban.deleteMany({ server: this._id }),
    AuditLog.deleteMany({ server: this._id }),
    Invite.deleteMany({ server: this._id }),
    Webhook.deleteMany({ server: this._id })
  ]);
  
  // Finally delete the server itself
  await this.deleteOne();
};

export default mongoose.model('Server', ServerSchema); 