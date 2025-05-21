import mongoose from 'mongoose';
import utility from '../../utils';

/**
 * Schema for channel categories
 * Organizes channels within servers
 */
const CategorySchema = new mongoose.Schema({
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
      message: 'Category name can only contain letters, numbers, spaces, hyphens, and underscores'
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
  position: {
    type: Number,
    required: true,
    min: 0
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
  voiceChannels: [{
    type: String,
    ref: 'VoiceChannel',
    validate: {
      validator: function(v) {
        return mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid voice channel reference'
    }
  }],
  isCollapsed: {
    type: Boolean,
    default: false
  },
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
CategorySchema.index({ server: 1, position: 1 });
CategorySchema.index({ server: 1, name: 1 });
CategorySchema.index({ isPrivate: 1 });

// Update timestamp before saving
CategorySchema.pre('save', function(next) {
  this.updatedAt = new Date();
  next();
});

// Method to check if user can view the category
CategorySchema.methods.canUserView = function(userRoles) {
  if (this.isPrivate) {
    if (!this.allowedRoles.length) return false;
    return userRoles.some(role => this.allowedRoles.includes(role));
  }
  return true;
};

// Method to get all channels in the category
CategorySchema.methods.getAllChannels = function() {
  return [...this.channels, ...this.voiceChannels];
};

export default mongoose.model('Category', CategorySchema); 