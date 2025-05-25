import mongoose from 'mongoose';
import { utility } from '../../utils.js';

/**
 * Schema for audit logs
 * Tracks important actions within servers
 */
const AuditLogSchema = new mongoose.Schema({
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
  action: {
    type: String,
    required: true,
    enum: [
      'MEMBER_JOIN',
      'MEMBER_LEAVE',
      'MEMBER_KICK',
      'MEMBER_BAN',
      'MEMBER_UNBAN',
      'MEMBER_ROLE_UPDATE',
      'CHANNEL_CREATE',
      'CHANNEL_DELETE',
      'CHANNEL_UPDATE',
      'ROLE_CREATE',
      'ROLE_DELETE',
      'ROLE_UPDATE',
      'SERVER_UPDATE',
      'WEBHOOK_CREATE',
      'WEBHOOK_DELETE',
      'WEBHOOK_UPDATE',
      'INVITE_CREATE',
      'INVITE_DELETE',
      'MESSAGE_DELETE',
      'MESSAGE_EDIT'
    ]
  },
  target: {
    type: String,
    refPath: 'targetModel',
    validate: {
      validator: function(v) {
        return mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid target reference'
    }
  },
  targetModel: {
    type: String,
    enum: ['User', 'Channel', 'Role', 'Message', 'Webhook', 'Invite']
  },
  changes: {
    type: Map,
    of: mongoose.Schema.Types.Mixed
  },
  reason: {
    type: String,
    maxlength: 1000,
    trim: true
  },
  createdAt: {
    type: Date,
    default: Date.now
  }
}, {
  timestamps: true
});

// Create indexes
AuditLogSchema.index({ server: 1, createdAt: -1 });
AuditLogSchema.index({ user: 1, createdAt: -1 });
AuditLogSchema.index({ action: 1, createdAt: -1 });
AuditLogSchema.index({ target: 1, targetModel: 1 });

// Method to format changes for display
AuditLogSchema.methods.formatChanges = function() {
  const formatted = {};
  if (this.changes) {
    this.changes.forEach((value, key) => {
      formatted[key] = {
        old: value.old,
        new: value.new
      };
    });
  }
  return formatted;
};

// Static method to get audit logs for a server
AuditLogSchema.statics.getServerLogs = async function(serverId, options = {}) {
  const query = { server: serverId };
  if (options.action) query.action = options.action;
  if (options.user) query.user = options.user;
  if (options.target) query.target = options.target;
  if (options.targetModel) query.targetModel = options.targetModel;
  
  return this.find(query)
    .populate('user', 'username avatar')
    .populate('target')
    .sort({ createdAt: -1 })
    .limit(options.limit || 100)
    .skip(options.skip || 0);
};

// Static method to get audit logs for a user
AuditLogSchema.statics.getUserLogs = async function(userId, options = {}) {
  const query = { user: userId };
  if (options.action) query.action = options.action;
  if (options.target) query.target = options.target;
  
  return this.find(query)
    .populate('server', 'name icon')
    .populate('target')
    .sort({ createdAt: -1 })
    .limit(options.limit || 100)
    .skip(options.skip || 0);
};

// Static method to get audit logs for a target
AuditLogSchema.statics.getTargetLogs = async function(targetId, targetModel, options = {}) {
  const query = { target: targetId, targetModel };
  if (options.action) query.action = options.action;
  if (options.user) query.user = options.user;
  
  return this.find(query)
    .populate('user', 'username avatar')
    .populate('server', 'name icon')
    .sort({ createdAt: -1 })
    .limit(options.limit || 100)
    .skip(options.skip || 0);
};

// Static method to delete old audit logs
AuditLogSchema.statics.deleteOldLogs = async function(daysToKeep = 90) {
  const cutoffDate = new Date();
  cutoffDate.setDate(cutoffDate.getDate() - daysToKeep);
  
  return this.deleteMany({ createdAt: { $lt: cutoffDate } });
};

// Static method to create an audit log entry
AuditLogSchema.statics.createLog = async function(data) {
  return this.create({
    server: data.server,
    user: data.user,
    action: data.action,
    target: data.target,
    targetModel: data.targetModel,
    changes: data.changes,
    reason: data.reason
  });
};

export default mongoose.model('AuditLog', AuditLogSchema); 