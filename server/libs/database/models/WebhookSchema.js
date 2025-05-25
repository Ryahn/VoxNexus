import mongoose from 'mongoose';
import { utility } from '../../utils.js';
import crypto from 'crypto';

/**
 * Schema for webhooks
 * Manages server webhooks for external integrations
 */
const WebhookSchema = new mongoose.Schema({
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
      message: 'Webhook name can only contain letters, numbers, spaces, hyphens, and underscores'
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
  channel: {
    type: String,
    ref: 'Channel',
    required: true,
    validate: {
      validator: function(v) {
        return mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid channel reference'
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
  token: {
    type: String,
    required: true,
    unique: true
  },
  avatar: {
    type: String,
    ref: 'Attachment',
    validate: {
      validator: function(v) {
        return !v || mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid attachment reference'
    }
  },
  isActive: {
    type: Boolean,
    default: true
  },
  lastUsed: {
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
WebhookSchema.index({ server: 1, channel: 1 });
WebhookSchema.index({ createdBy: 1 });
WebhookSchema.index({ isActive: 1 });
WebhookSchema.index({ lastUsed: 1 });

// Generate webhook token before saving
WebhookSchema.pre('save', function(next) {
  if (!this.token) {
    this.token = crypto.randomBytes(32).toString('hex');
  }
  this.updatedAt = new Date();
  next();
});

// Method to generate webhook URL
WebhookSchema.methods.getWebhookUrl = function() {
  return `/api/webhooks/${this._id}/${this.token}`;
};

// Method to verify webhook token
WebhookSchema.methods.verifyToken = function(token) {
  return this.token === token;
};

// Method to update last used timestamp
WebhookSchema.methods.updateLastUsed = function() {
  this.lastUsed = new Date();
  return this.save();
};

export default mongoose.model('Webhook', WebhookSchema); 