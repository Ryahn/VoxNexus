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

// Method to add a text channel
CategorySchema.methods.addTextChannel = async function(channelId) {
  if (!this.channels.includes(channelId)) {
    this.channels.push(channelId);
    await this.save();
  }
};

// Method to add a voice channel
CategorySchema.methods.addVoiceChannel = async function(channelId) {
  if (!this.voiceChannels.includes(channelId)) {
    this.voiceChannels.push(channelId);
    await this.save();
  }
};

// Method to remove a text channel
CategorySchema.methods.removeTextChannel = async function(channelId) {
  this.channels = this.channels.filter(c => c.toString() !== channelId.toString());
  await this.save();
};

// Method to remove a voice channel
CategorySchema.methods.removeVoiceChannel = async function(channelId) {
  this.voiceChannels = this.voiceChannels.filter(c => c.toString() !== channelId.toString());
  await this.save();
};

// Method to update position
CategorySchema.methods.updatePosition = async function(newPosition) {
  if (newPosition >= 0) {
    this.position = newPosition;
    await this.save();
  }
};

// Method to toggle collapse state
CategorySchema.methods.toggleCollapse = async function() {
  this.isCollapsed = !this.isCollapsed;
  await this.save();
};

// Method to toggle privacy
CategorySchema.methods.togglePrivacy = async function() {
  this.isPrivate = !this.isPrivate;
  await this.save();
};

// Method to add allowed role
CategorySchema.methods.addAllowedRole = async function(roleId) {
  if (!this.allowedRoles.includes(roleId)) {
    this.allowedRoles.push(roleId);
    await this.save();
  }
};

// Method to remove allowed role
CategorySchema.methods.removeAllowedRole = async function(roleId) {
  this.allowedRoles = this.allowedRoles.filter(r => r.toString() !== roleId.toString());
  await this.save();
};

// Method to update allowed roles
CategorySchema.methods.updateAllowedRoles = async function(roleIds) {
  this.allowedRoles = roleIds;
  await this.save();
};

// Static method to get categories by server
CategorySchema.statics.getServerCategories = async function(serverId) {
  return this.find({ server: serverId })
    .populate('channels', 'name type')
    .populate('voiceChannels', 'name')
    .populate('allowedRoles', 'name color')
    .sort({ position: 1 });
};

// Static method to get private categories
CategorySchema.statics.getPrivateCategories = async function(serverId, userRoles) {
  return this.find({
    server: serverId,
    isPrivate: true,
    allowedRoles: { $in: userRoles }
  })
    .populate('channels', 'name type')
    .populate('voiceChannels', 'name')
    .populate('allowedRoles', 'name color')
    .sort({ position: 1 });
};

// Static method to get category by name
CategorySchema.statics.getCategoryByName = async function(serverId, name) {
  return this.findOne({ server: serverId, name })
    .populate('channels', 'name type')
    .populate('voiceChannels', 'name')
    .populate('allowedRoles', 'name color');
};

// Static method to reorder categories
CategorySchema.statics.reorderCategories = async function(serverId, categoryIds) {
  const updates = categoryIds.map((id, index) => ({
    updateOne: {
      filter: { _id: id, server: serverId },
      update: { position: index }
    }
  }));
  
  return this.bulkWrite(updates);
};

// Static method to create category
CategorySchema.statics.createCategory = async function(data) {
  return this.create({
    name: data.name,
    server: data.server,
    position: data.position,
    isPrivate: data.isPrivate,
    allowedRoles: data.allowedRoles
  });
};

export default mongoose.model('Category', CategorySchema); 